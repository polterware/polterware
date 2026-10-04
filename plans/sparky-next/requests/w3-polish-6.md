# W3-POLISH request 6: link title fetching preference

Owner: W1-ATTACH / integrator. Status: open.

The attachment provider exposes FetchTitle, but no production call site in sparky/backend invokes it (rg -n FetchTitle sparky/backend shows the declaration and tests only). links.fetchTitles round-trips and the previous UI claimed it would request the URL automatically. W3-POLISH changes that copy to state that automatic fetching is unavailable and that the person can enter a title.

Requested behavior: wire the opt-in preference into AddLink with the existing bounded SSRF-safe fetcher, then revise the UI only after real request/failure/disabled tests pass. Alternatively remove the unusable preference through an approved product decision. No provider or contract code changed in W3-POLISH.
