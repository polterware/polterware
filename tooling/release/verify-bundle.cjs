// Verifies a packaged desktop app: node verify-bundle.cjs <app> <path/to/App.app>
const assert = require('node:assert/strict');
const { execFileSync, spawnSync } = require('node:child_process');
const { existsSync, readFileSync, readdirSync, statSync } = require('node:fs');
const { createRequire } = require('node:module');
const { join, resolve } = require('node:path');
const { releaseApp } = require('./apps.cjs');

const release = releaseApp(process.argv[2]);
const desktop = resolve(__dirname, '../..', release.desktop);
const builderRequire = createRequire(require.resolve('electron-builder', { paths: [desktop] }));
const asar = createRequire(builderRequire.resolve('app-builder-lib'))('@electron/asar');
const app = resolve(process.argv[3]);
const resources = join(app, 'Contents/Resources');

execFileSync('codesign', ['--verify', '--deep', '--strict', app], { stdio: 'inherit' });
// codesign prints signature metadata to stderr.
const signature = spawnSync('codesign', ['--display', '--verbose=4', app], { encoding: 'utf8' });
assert.match(signature.stderr, /Signature=adhoc/);
assert.doesNotMatch(signature.stderr, /flags=.*runtime/);

for (const { path, arch } of release.binaries) {
  const binary = join(resources, path);
  assert.ok(existsSync(binary), `${path} missing`);
  assert.equal(execFileSync('lipo', ['-archs', binary], { encoding: 'utf8' }).trim(), arch, `${path} architecture`);
  execFileSync('codesign', ['--verify', '--strict', binary]);
}

const plist = JSON.parse(execFileSync('plutil', ['-convert', 'json', '-o', '-', join(app, 'Contents/Info.plist')], { encoding: 'utf8' }));
assert.equal(plist.CFBundleIdentifier, release.appId);
for (const scheme of release.urlSchemes) {
  assert.ok((plist.CFBundleURLTypes ?? []).some((type) => type.CFBundleURLSchemes.includes(scheme)), `${scheme}:// missing`);
}
for (const key of release.plistKeys) assert.ok(plist[key], `${key} missing`);

const archive = join(resources, 'app.asar');
const entries = asar.listPackage(archive);
assert.ok(entries.includes('/out/renderer/index.html'));
if (release.unpackedRenderer) {
  for (const name of entries.filter((entry) => entry.startsWith('/out/renderer/') && /\.(html|js|css|woff2|png|webp|svg|webm)$/.test(entry))) {
    assert.ok(asar.statFile(archive, name.slice(1)).unpacked, `${name} must be unpacked`);
    assert.ok(statSync(join(`${archive}.unpacked`, name)).isFile());
  }
}

const source = JSON.parse(readFileSync(join(desktop, 'package.json'), 'utf8'));
const devOnly = Object.keys(source.devDependencies ?? {}).filter((name) => !(source.dependencies ?? {})[name]);
for (const name of entries) {
  assert.ok(!name.endsWith('.map'), `Source map shipped: ${name}`);
  for (const dep of devOnly) assert.ok(!name.includes(`/node_modules/${dep}/`), `Development dependency shipped: ${dep}`);
}
(function checkPhysical(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) checkPhysical(path);
    else assert.ok(!entry.name.endsWith('.map'), `Physical source map shipped: ${path}`);
  }
})(resources);
assert.ok(!existsSync(join(resources, 'app-update.yml')), 'Automatic update feed must stay disabled');

console.log(`PASS ${release.product}: ad-hoc signature, strict deep verification, hardened runtime off, nested binaries (${release.binaries.map((b) => `${b.path}:${b.arch}`).join(', ')}), bundle id, no source maps, dev-only dependencies or update feed`);
