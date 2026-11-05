const { test } = require('node:test');
const assert = require('node:assert/strict');
const { plist } = require('./gpui-plist.cjs');
const config = require('../updater/config.cjs');

test('GPUI bundle uses the Electron release identity and Sparkle configuration', () => {
  const xml = plist('0.1.1', '123456');
  for (const [key, value] of Object.entries({ CFBundleIdentifier: 'com.polterware.polterware', CFBundleName: 'Polterware', CFBundleExecutable: 'Polterware', CFBundleShortVersionString: '0.1.1', CFBundleVersion: '123456', SUFeedURL: config.feedUrl('polterware'), SUPublicEDKey: config.publicKey })) {
    assert.ok(xml.includes(`<key>${key}</key><string>${value}</string>`));
  }
  assert.match(xml, /<key>SUScheduledCheckInterval<\/key><integer>86400<\/integer>/);
  assert.match(xml, /<key>SUEnableAutomaticChecks<\/key><true\/>/);
  assert.throws(() => plist('bad', '1'));
  assert.throws(() => plist('1.2.3', 'bad'));
});
