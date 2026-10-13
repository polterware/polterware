// Release metadata for the macOS desktop apps. dist-mac.sh, sign-mac.cjs,
// verify-bundle.cjs and the desktop-release workflow read it so Lore, Sparky
// and Vitals package, sign and publish the same way.
const electron = '44.4.5';

const apps = {
  lore: {
    product: 'Lore',
    appId: 'com.polterware.lore',
    desktop: 'apps/lore-desktop',
    goMod: 'backend/go.mod',
    // Mach-O files under Contents/Resources, signed before the app itself.
    binaries: [{ path: 'polterware', arch: 'arm64' }],
    urlSchemes: [],
    plistKeys: [],
    unpackedRenderer: false,
  },
  sparky: {
    product: 'Sparky',
    appId: 'com.polterware.sparky',
    desktop: 'apps/sparky-desktop',
    goMod: 'sparky/backend/go.mod',
    binaries: [
      { path: 'sparky', arch: 'arm64' },
      { path: 'native/notify.node', arch: 'arm64' },
    ],
    urlSchemes: ['sparky'],
    plistKeys: ['NSMicrophoneUsageDescription'],
    unpackedRenderer: true,
  },
  vitals: {
    product: 'Vitals',
    appId: 'com.polterware.vitals',
    desktop: 'apps/vitals',
    goMod: 'apps/vitals/agent/go.mod',
    // Agents uploaded to remote macOS hosts, so both architectures ship.
    binaries: [
      { path: 'agent/vitals-agent-darwin-arm64', arch: 'arm64' },
      { path: 'agent/vitals-agent-darwin-amd64', arch: 'x86_64' },
    ],
    urlSchemes: [],
    plistKeys: [],
    unpackedRenderer: false,
  },
};

function releaseApp(key) {
  const app = apps[key];
  if (!app) throw new Error(`Unknown release app: ${key} (expected ${Object.keys(apps).join(', ')})`);
  return { key, ...app };
}

function releaseAppById(appId) {
  const key = Object.keys(apps).find((name) => apps[name].appId === appId);
  if (!key) throw new Error(`No release metadata for ${appId}`);
  return releaseApp(key);
}

module.exports = { electron, apps, releaseApp, releaseAppById };

// `node apps.cjs <app> <field>` prints one field for shell scripts and CI.
if (require.main === module) {
  const [key, field] = process.argv.slice(2);
  const value = releaseApp(key)[field];
  if (value === undefined) throw new Error(`Unknown field: ${field}`);
  process.stdout.write(typeof value === 'string' ? value : JSON.stringify(value));
}
