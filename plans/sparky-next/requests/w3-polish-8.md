# W3-POLISH request 8: preserve native control keyboard activation

Owner: W2-SHELL / W2-MEMORIES / W2-SEARCH. Status: open.

Browser reproduction: navigate Today, focus the Open Commander button, press Enter. No dialog opens. The shell's document key handler maps Enter to memoryShortcut("open") and calls preventDefault whenever a collection handler is registered, even on titlebar/sidebar buttons and with no active memory. Settings and ordinary buttons are affected similarly. SearchPage also prevents Enter/arrow defaults at its root for non-input controls, so Save filters and filters need the same audit.

Requested behavior: dispatch collection shortcuts only from the collection/row context, exempt native buttons/links/checkboxes/radios/menu items, and preventDefault only when an action is handled. Preserve native Enter/Space activation and Ark navigation. Add browser/DOM tests for Enter on Commander, Settings, New memory, filters and row actions plus row shortcut parity. W3-POLISH records the failure rather than changing application shortcut behavior.
