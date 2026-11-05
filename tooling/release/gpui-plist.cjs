// Bundle metadata follows Electron's apps.cjs and updater/package.cjs.
const release = require('./apps.cjs').releaseApp('polterware');
const updater = require('../updater/config.cjs');

function plist(version, build) {
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version) || !/^\d+$/.test(build)) throw Error('Invalid version or build number');
  const values = {
    CFBundleIdentifier: release.appId, CFBundleName: release.product, CFBundleDisplayName: release.product,
    CFBundleExecutable: release.product, CFBundlePackageType: 'APPL', CFBundleShortVersionString: version,
    CFBundleVersion: build, CFBundleIconName: 'polterware', CFBundleIconFile: 'polterware',
    LSMinimumSystemVersion: '15.0', LSApplicationCategoryType: 'public.app-category.developer-tools',
    SUFeedURL: updater.feedUrl('polterware'), SUPublicEDKey: updater.publicKey,
  };
  const escape = value => value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
  const entries = Object.entries(values).map(([key, value]) => `  <key>${key}</key><string>${escape(value)}</string>`);
  entries.push('  <key>NSHighResolutionCapable</key><true/>', '  <key>SUEnableAutomaticChecks</key><true/>',
    '  <key>SUScheduledCheckInterval</key><integer>86400</integer>', '  <key>SUAutomaticallyUpdate</key><false/>',
    '  <key>SUShowReleaseNotes</key><false/>');
  return `<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n<plist version="1.0"><dict>\n${entries.join('\n')}\n</dict></plist>\n`;
}
module.exports = { plist };
if (require.main === module) process.stdout.write(plist(process.argv[2], process.argv[3]));
