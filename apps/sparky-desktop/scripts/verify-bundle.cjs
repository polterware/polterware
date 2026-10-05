const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const { existsSync, readFileSync, readdirSync, statSync } = require('node:fs');
const { createRequire } = require('node:module');
const { join, resolve } = require('node:path');
const desktop = resolve(__dirname, '..');
const builderRequire = createRequire(require.resolve('electron-builder', { paths: [desktop] }));
const asar = createRequire(builderRequire.resolve('app-builder-lib'))('@electron/asar');
const app = resolve(process.argv[2]);
const resources = join(app, 'Contents/Resources');
execFileSync('codesign', ['--verify', '--deep', '--strict', app], { stdio: 'inherit' });
// codesign emits signature metadata to stderr; read it separately.
const result = require('node:child_process').spawnSync('codesign', ['--display', '--verbose=4', app], { encoding: 'utf8' });
assert.match(result.stderr, /Signature=adhoc/);
assert.doesNotMatch(result.stderr, /flags=.*runtime/);
for (const name of ['sparky', 'native/notify.node']) {
  const binary = join(resources, name);
  assert.ok(existsSync(binary), `${name} missing`);
  assert.equal(execFileSync('lipo', ['-archs', binary], { encoding: 'utf8' }).trim(), 'arm64');
  execFileSync('codesign', ['--verify', '--strict', binary]);
}
const plist = JSON.parse(execFileSync('plutil', ['-convert', 'json', '-o', '-', join(app, 'Contents/Info.plist')], { encoding: 'utf8' }));
assert.equal(plist.CFBundleIdentifier, 'com.polterware.sparky');
assert.ok(plist.CFBundleURLTypes.some(type => type.CFBundleURLSchemes.includes('sparky')));
assert.ok(plist.NSMicrophoneUsageDescription);
const archive = join(resources, 'app.asar');
const entries = asar.listPackage(archive);
assert.ok(entries.includes('/out/renderer/index.html'));
for (const name of entries.filter(name => name.startsWith('/out/renderer/') && /\.(html|js|css|woff2|png|webp|svg|webm)$/.test(name))) {
  assert.ok(asar.statFile(archive, name.slice(1)).unpacked, `${name} must be unpacked`);
  assert.ok(statSync(join(`${archive}.unpacked`, name)).isFile());
}
const source = JSON.parse(readFileSync(join(desktop, 'package.json'), 'utf8'));
const devOnly = Object.keys(source.devDependencies).filter(name => !source.dependencies[name]);
for (const name of entries) {
  assert.ok(!name.endsWith('.map'), `Source map shipped: ${name}`);
  for (const dep of devOnly) assert.ok(!name.includes(`/node_modules/${dep}/`), `Development dependency shipped: ${dep}`);
}
function checkPhysical(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) checkPhysical(path);
    else assert.ok(!entry.name.endsWith('.map'), `Physical source map shipped: ${path}`);
  }
}
checkPhysical(resources);
assert.ok(!existsSync(join(resources, 'app-update.yml')), 'Automatic update feed must stay disabled');
console.log('PASS: ad-hoc signature, strict deep verification, hardened runtime off, arm64 core/addon, sparky://, microphone usage description, physical renderer, no source maps/dev-only dependencies/update feed');
