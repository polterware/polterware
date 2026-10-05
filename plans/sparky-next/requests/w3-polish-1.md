# W3-POLISH request 1: calendar keyboard navigation

Owner: W2-CALENDAR / integrator. Status: open.

The page-level keyboard handler consumes arrows from view radios, date-picker buttons and occurrence buttons. It changes the anchor/highlight but leaves DOM focus on the original control. Every hourly slot is also a separate Tab stop (over 168 in week view). CalendarGrid exposes regions, not a calendar grid navigation model.

Requested behavior change: limit day navigation to a dedicated grid; use roving focus with full date names, move actual focus across dates/slots, compose with Ark radio/date-picker keyboard behavior, and preserve Enter/Space activation. Add behavior tests for month boundaries, week/day slots, radio arrows and date-picker popup arrows. Do not add grid roles without the complete focus model.

W3-POLISH only adds date names, visible focus and motion styling. Keyboard acceptance remains open until the owner implements the navigation model.
