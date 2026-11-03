// Release metadata for the macOS desktop app. dist-mac.sh, sign-mac.cjs,
// verify-bundle.cjs and the desktop-release workflow read it.
const electron = '44.4.5';

const apps = {
  polterware: {
    product: 'Polterware',
    appId: 'com.polterware.polterware',
    desktop: 'apps/polterware-desktop',
    goMod: 'backend/go.mod',
    // Mach-O files under Contents/Resources, signed before the app itself.
    binaries: [{ path: 'polterware', arch: 'arm64' }],
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
