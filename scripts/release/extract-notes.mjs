import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

export function extractReleaseNotes(changelog, version) {
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`Invalid semantic version: ${version}`);
  }
  const lines = changelog.replace(/^\uFEFF/, '').split(/\r?\n/);
  const headings = lines.map((line, index) => ({ line, index })).filter(({ line }) => /^##\s/.test(line));
  const matches = headings.filter(({ line }) => line.trim() === `## ${version}`);
  if (matches.length === 0) throw new Error(`No changelog section for ${version}.`);
  if (matches.length > 1) throw new Error(`Duplicate changelog sections for ${version}.`);
  const [section] = matches;
  const next = headings.find(({ index }) => index > section.index);
  const notes = lines.slice(section.index + 1, next?.index ?? lines.length).join('\n').trim();
  if (!notes) throw new Error(`The changelog section for ${version} is empty.`);
  return `${notes.replace(/\r\n?/g, '\n')}\n`;
}

async function main() {
  const argIndex = process.argv.indexOf('--version');
  const tag = argIndex >= 0 ? process.argv[argIndex + 1] : process.env.GITHUB_REF_NAME;
  const version = tag?.replace(/^v/, '');
  if (!version) throw new Error('Supply --version <semver> or GITHUB_REF_NAME.');
  const changelog = await readFile(new URL('../../CHANGELOG.md', import.meta.url), 'utf8');
  const notes = extractReleaseNotes(changelog, version);
  await writeFile(new URL('../../release-notes.md', import.meta.url), notes, { encoding: 'utf8', flag: 'w' });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main().catch((error) => { console.error(error.message); process.exitCode = 1; });
}
