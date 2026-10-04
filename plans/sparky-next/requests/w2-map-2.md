---
summary: Add map.geocoding setting and Nominatim CSP host
rationale: Address search is opt-in and disabled by default (A9 / O4). settings.proto has no geocoding key, and Main CSP only allows tiles.openfreemap.org when map.online is true. W2-MAP must not edit Main or settings.proto.
---

W2-MAP requests a contracts + Electron follow-up (W2-SETTINGS / W1-ELECTRON / Erick):

1. Add a bool setting `map.geocoding` (default **false**) to `Settings` and the `map.geocoding` wire key. Do not enable it by the `map.online` disclosure. Nominatim is end-user triggered only, max 1 req/s, with a real User-Agent.
2. When `map.geocoding` is true, widen renderer CSP `connect-src` with `https://nominatim.openstreetmap.org` (and only then). Do not add this host while the setting is off.
3. Keep the existing O4 `map.online` CSP rule (`connect-src https://tiles.openfreemap.org`, `worker-src 'self' blob:`, `img-src 'self' data: blob:`). No change requested there.

W2-MAP ships a disabled-by-default geocoding adapter (`createDisabledGeocoder` / `createNominatimGeocoder`) that **never fetches** unless a future `mapGeocoding` field is true. The Map settings panel states that address search is off until this setting and CSP host exist.

W2-MAP does not edit `sparky/schemas/**`, `apps/sparky-desktop/src/main/**`, or `settings.proto`.
