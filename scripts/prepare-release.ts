// Bumps the version for a release: `version` in Cargo.toml, the crate's entry in Cargo.lock, and
// turns the CHANGELOG's [Unreleased] section into the new version's section. The Prepare
// release workflow runs it; it also works locally.
//
//   node scripts/prepare-release.ts <patch|minor|major|X.Y.Z>
//
// Prints the new version. Fails without touching anything if the version doesn't go up or
// [Unreleased] is empty.
import { readFileSync, writeFileSync } from 'node:fs';

type Version = [number, number, number];

function fail(message: string): never {
  console.error(`prepare-release: ${message}`);
  process.exit(1);
}

function parse(text: string): Version | undefined {
  const m = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.exec(text);
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : undefined;
}

function isNewer(a: Version, b: Version): boolean {
  for (let i = 0; i < 3; i++) if (a[i] !== b[i]) return a[i] > b[i];
  return false;
}

function next(current: Version, arg: string): Version {
  const [major, minor, patch] = current;
  switch (arg) {
    case 'major':
      return [major + 1, 0, 0];
    case 'minor':
      return [major, minor + 1, 0];
    case 'patch':
      return [major, minor, patch + 1];
  }
  const explicit = parse(arg.replace(/^v/, '')) ?? fail(`expected patch, minor, major or X.Y.Z, got "${arg}"`);
  if (!isNewer(explicit, current)) fail(`${explicit.join('.')} is not newer than ${current.join('.')}`);
  return explicit;
}

const arg = process.argv[2] ?? fail('usage: node scripts/prepare-release.ts <patch|minor|major|X.Y.Z>');

// Same pattern the Release workflow uses to check the tag against the crate version.
const versionLine = /^version = "(.+)"$/m;
const cargoToml = readFileSync('Cargo.toml', 'utf8');
const currentText = versionLine.exec(cargoToml)?.[1] ?? fail('no version in Cargo.toml');
const current = parse(currentText) ?? fail(`Cargo.toml version "${currentText}" is not X.Y.Z`);
const version = next(current, arg).join('.');

const lockEntry = `name = "no-drama-llama"\nversion = "${currentText}"\n`;
const cargoLock = readFileSync('Cargo.lock', 'utf8').replace(/\r\n/g, '\n');
if (!cargoLock.includes(lockEntry)) fail(`Cargo.lock has no no-drama-llama ${currentText} entry`);

const changelog = readFileSync('CHANGELOG.md', 'utf8').replace(/\r\n/g, '\n');
const unreleased = /^## \[Unreleased\]\n([\s\S]*?)(?=^## \[)/m.exec(changelog) ?? fail('no ## [Unreleased] section in CHANGELOG.md');
if (!unreleased[1].trim()) fail('the CHANGELOG [Unreleased] section is empty: nothing to release');
if (changelog.includes(`## [${version}]`)) fail(`CHANGELOG.md already has a ${version} section`);

writeFileSync('Cargo.toml', cargoToml.replace(versionLine, `version = "${version}"`));
writeFileSync('Cargo.lock', cargoLock.replace(lockEntry, `name = "no-drama-llama"\nversion = "${version}"\n`));
writeFileSync(
  'CHANGELOG.md',
  changelog.replace(unreleased[0], `## [Unreleased]\n\n## [${version}]\n${unreleased[1]}`),
);
console.log(version);
