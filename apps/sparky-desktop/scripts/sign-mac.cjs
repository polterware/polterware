const { execFileSync } = require('node:child_process');
const { join } = require('node:path');
const { statSync, utimesSync } = require('node:fs');

// afterPack runs after resources/plist assembly and before DMG/ZIP creation.
module.exports = async ({ appOutDir, electronPlatformName }) => {
  if (electronPlatformName !== 'darwin') throw new Error('Sparky releases are macOS only');
  const app = join(appOutDir, 'Sparky.app');
  const resources = join(app, 'Contents', 'Resources');
  const binaries = [join(resources, 'sparky'), join(resources, 'native', 'notify.node')];
  const times = binaries.map(binary => statSync(binary));
  for (const binary of binaries) {
    execFileSync('codesign', ['--force', '--sign', '-', '--timestamp=none', binary], { stdio: 'inherit' });
  }
  // No hardened runtime, certificates or entitlements are used. Microphone needs
  // the Info.plist usage description; UserNotifications needs the bundle identity.
  execFileSync('codesign', ['--force', '--deep', '--sign', '-', '--timestamp=none', app], { stdio: 'inherit' });
  // Deep signing also rewrites nested binaries. Restore mtime only after it so
  // the core build handshake follows SOURCE_DATE_EPOCH rather than wall time.
  binaries.forEach((binary, index) => utimesSync(binary, times[index].atime, times[index].mtime));
  execFileSync('codesign', ['--verify', '--deep', '--strict', app], { stdio: 'inherit' });
};
