import assert from 'node:assert/strict';
import { extractReleaseNotes } from './extract-notes.mjs';

let passed = 0;
function test(name, callback) {
  callback();
  passed += 1;
  console.log(`PASS ${name}`);
}

const changelog = `# Changelog\n\n## 1.2.0\n\n- First\n\n## 1.1.0\n\n- Middle\n\n## 1.0.0\n\n- Last\n`;

test('first, middle, and last sections', () => {
  assert.equal(extractReleaseNotes(changelog, '1.2.0'), '- First\n');
  assert.equal(extractReleaseNotes(changelog, '1.1.0'), '- Middle\n');
  assert.equal(extractReleaseNotes(changelog, '1.0.0'), '- Last\n');
});

test('missing version and empty section rejection', () => {
  assert.throws(() => extractReleaseNotes(changelog, '2.0.0'), /No changelog section/);
  assert.throws(() => extractReleaseNotes('# Changelog\n\n## 2.0.0\n\n## 1.0.0\n- Old', '2.0.0'), /empty/);
});

test('duplicate versions and malformed semantic version rejection', () => {
  assert.throws(() => extractReleaseNotes(`${changelog}\n## 1.0.0\n- Duplicate`, '1.0.0'), /Duplicate/);
  assert.throws(() => extractReleaseNotes(changelog, '../1.0.0'), /Invalid semantic version/);
});

test('following release section is excluded', () => {
  const notes = extractReleaseNotes(changelog, '1.1.0');
  assert.equal(notes, '- Middle\n');
  assert.doesNotMatch(notes, /Last/);
});

console.log(`${passed} release-note extraction test groups passed.`);
