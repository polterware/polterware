# W3-POLISH request 2: theme, contrast and app motion preference

Owner: W0-SCAFFOLD / W1-ELECTRON / contracts owner. Status: open.

The frozen Settings contract has appearanceTheme and the Main-owned Reduce transparency preference, but no Reduce motion setting. Neither renderer nor Sparky feature code applies appearanceTheme to a light token palette. The shared root token file defines dark tokens only. A light dropdown value is currently stored without a visible theme change.

The dark text-muted token (#6f6f6f) fails WCAG AA normal text on surface (#0f0f0f, about 3.8:1) and selected (#232323, about 3.1:1). The border-strong alpha token is not a sufficient focus indicator; W3-POLISH improves shared focus styling using existing opaque text tokens. Shared empty states and table headers still need corrected root muted tokens.

Requested change: provide an approved opaque light palette and accessible dark muted token in packages/configuration/tailwind/base.css, apply theme selection in the renderer host, and define/persist an app Reduce motion preference with a root attribute/DS adapter so CSS and JS motion use OS OR app preferences. This requires behavior/host work and possibly an additive contract change; do not fake the setting in W3-POLISH. Retest all DS controls, status pairs, focus outlines, transparency and motion in both palettes.

ProgressRing also uses an SVG stroke-dashoffset transition rather than the requested transform-only progress. Resolve the shared progress strategy with the motion owner and verify frame cost before claiming that requirement complete.
