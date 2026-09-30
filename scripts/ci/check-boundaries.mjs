import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';
import { loadConfigFromFile, resolveConfig } from 'vite';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const frontendRoot = join(repositoryRoot, 'src');
const forbiddenModules = new Set([
  'child_process',
  'node:child_process',
  '@tauri-apps/plugin-shell',
  '@tauri-apps/plugin-http',
  'axios',
  'cross-fetch',
  'got',
  'node-fetch',
  'superagent',
  'undici',
]);
const forbiddenGlobals = new Set(['fetch', 'XMLHttpRequest', 'WebSocket', 'sendBeacon']);

async function sourceFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(entries.map(async (entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    return ['.ts', '.tsx', '.js', '.jsx'].includes(extname(entry.name)) ? [path] : [];
  }));
  return nested.flat();
}

function isForbiddenNetworkCall(expression) {
  if (ts.isIdentifier(expression)) return forbiddenGlobals.has(expression.text);
  if (ts.isPropertyAccessExpression(expression)) {
    if (['fetch', 'sendBeacon'].includes(expression.name.text)
      && ['window', 'globalThis', 'navigator'].includes(expression.expression.getText())) return true;
    return isForbiddenNetworkCall(expression.expression);
  }
  if (ts.isElementAccessExpression(expression)) {
    const property = expression.argumentExpression;
    if (property && ts.isStringLiteral(property)
      && ['fetch', 'sendBeacon'].includes(property.text)
      && ['window', 'globalThis', 'navigator'].includes(expression.expression.getText())) return true;
    return isForbiddenNetworkCall(expression.expression);
  }
  return false;
}

const violations = [];
for (const path of await sourceFiles(frontendRoot)) {
  const content = await readFile(path, 'utf8');
  const scriptKind = path.endsWith('.tsx') ? ts.ScriptKind.TSX
    : path.endsWith('.jsx') ? ts.ScriptKind.JSX
      : path.endsWith('.js') ? ts.ScriptKind.JS
        : ts.ScriptKind.TS;
  const source = ts.createSourceFile(path, content, ts.ScriptTarget.Latest, true, scriptKind);
  const report = (node, reason) => violations.push(`${path.slice(repositoryRoot.length + 1)}:${source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1}: ${reason}`);

  function visit(node) {
    if (ts.isImportDeclaration(node) && ts.isStringLiteral(node.moduleSpecifier) && forbiddenModules.has(node.moduleSpecifier.text)) {
      report(node, `frontend must not import execution-capability module ${node.moduleSpecifier.text}`);
    }
    if (ts.isCallExpression(node)
      && (node.expression.kind === ts.SyntaxKind.ImportKeyword || ts.isIdentifier(node.expression) && node.expression.text === 'require')
      && node.arguments[0]
      && ts.isStringLiteral(node.arguments[0])
      && forbiddenModules.has(node.arguments[0].text)) {
      report(node, `frontend must not load execution-capability module ${node.arguments[0].text}`);
    }
    if (ts.isExportDeclaration(node) && node.moduleSpecifier && ts.isStringLiteral(node.moduleSpecifier) && forbiddenModules.has(node.moduleSpecifier.text)) {
      report(node, `frontend must not re-export execution-capability module ${node.moduleSpecifier.text}`);
    }
    if (ts.isCallExpression(node) && isForbiddenNetworkCall(node.expression)) {
      report(node, 'frontend network access must remain behind validated backend IPC');
    }
    if (ts.isNewExpression(node) && ts.isIdentifier(node.expression) && ['XMLHttpRequest', 'WebSocket'].includes(node.expression.text)) {
      report(node, `frontend must not construct ${node.expression.text} directly`);
    }
    if (ts.isJsxAttribute(node) && node.name.getText(source) === 'dangerouslySetInnerHTML') {
      report(node, 'untrusted media/provider metadata must not be inserted as raw HTML');
    }
    if (ts.isBinaryExpression(node)
      && node.operatorToken.kind === ts.SyntaxKind.EqualsToken
      && ts.isPropertyAccessExpression(node.left)
      && ['innerHTML', 'outerHTML'].includes(node.left.name.text)) {
      report(node, 'untrusted media/provider metadata must not be inserted as raw HTML');
    }
    ts.forEachChild(node, visit);
  }

  visit(source);
}

assert.deepEqual(violations, [], `Frontend capability boundary violations:\n${violations.join('\n')}`);

const tauriConfig = JSON.parse(await readFile(join(repositoryRoot, 'src-tauri/tauri.conf.json'), 'utf8'));
const viteFile = join(repositoryRoot, 'vite.config.ts');
const loadedVite = await loadConfigFromFile({ command: 'serve', mode: 'development' }, viteFile);
assert.ok(loadedVite, 'Vite development configuration must load');
const vite = await resolveConfig(loadedVite.config, 'serve');
assert.equal(vite.server.strictPort, true, 'Vite must fail rather than silently selecting another port');

const devUrl = new URL(tauriConfig.build.devUrl);
assert.equal(devUrl.protocol, 'http:', 'Tauri development URL must use the local HTTP dev server');
assert.equal(devUrl.hostname, 'localhost', 'Tauri development URL must stay on localhost');
assert.equal(devUrl.port, String(vite.server.port), 'Tauri devUrl port must equal the Vite server port');

const pluginNames = new Set(vite.plugins.map((plugin) => plugin.name));
assert.ok([...pluginNames].some((name) => name.startsWith('@tailwindcss/vite:')),
  'Tailwind utility classes require the Tailwind Vite integration');
assert.match(await readFile(join(frontendRoot, 'index.css'), 'utf8'), /@import\s+["']tailwindcss["']/,
  'Tailwind Vite integration must have a Tailwind CSS entry point');

const csp = tauriConfig.app.security.csp;
const directives = new Map(csp.split(';').map((entry) => {
  const [name, ...values] = entry.trim().split(/\s+/);
  return [name, values];
}));
assert.ok(directives.get('script-src')?.includes("'self'"), 'CSP must restrict scripts to the application origin');
assert.ok(!directives.get('script-src')?.some((value) => ['*', "'unsafe-eval'"].includes(value)),
  'CSP must not allow wildcard or eval-based scripts');
assert.ok(directives.get('object-src')?.includes("'none'"), 'CSP must keep object embeds disabled');
assert.ok(directives.get('frame-src')?.includes("'none'"), 'CSP must keep frames disabled');
assert.ok(directives.get('connect-src')?.includes('ipc:'), 'CSP must retain Tauri IPC');
assert.ok(directives.get('connect-src')?.includes(`http://localhost:${vite.server.port}`),
  'CSP must allow the configured local development server');

const capabilitiesRoot = join(repositoryRoot, 'src-tauri/capabilities');
const capabilities = (await readdir(capabilitiesRoot)).filter((name) => name.endsWith('.json'));
assert.ok(capabilities.length > 0, 'At least one explicit Tauri capability file is required');
for (const name of capabilities) {
  const capability = JSON.parse(await readFile(join(capabilitiesRoot, name), 'utf8'));
  for (const permission of capability.permissions) {
    assert.ok(!/^(shell|fs):/.test(permission),
      `${name} must not grant arbitrary shell or filesystem plugin permission (${permission})`);
  }
}

console.log(`CI boundary checks passed (${violations.length} frontend capability violations).`);
