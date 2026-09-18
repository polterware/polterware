package localcore

import (
	"testing"
	"time"
)

func TestDecideScheduleTickTable(t *testing.T) {
	t.Parallel()
	now := time.Date(2026, 9, 18, 15, 0, 10, 0, time.UTC)
	cases := []struct {
		name     string
		nextRun  string
		wantAct  scheduleTickAction
		wantNext string
	}{
		{name: "empty advances", nextRun: "", wantAct: scheduleTickAdvance, wantNext: "2026-09-18T16:00:00Z"},
		{name: "near due fires", nextRun: "2026-09-18T15:00:00Z", wantAct: scheduleTickFire, wantNext: "2026-09-18T16:00:00Z"},
		{name: "far past skips", nextRun: "2026-09-18T10:00:00Z", wantAct: scheduleTickAdvance, wantNext: "2026-09-18T16:00:00Z"},
		{name: "future waits", nextRun: "2026-09-18T16:00:00Z", wantAct: scheduleTickWait, wantNext: "2026-09-18T16:00:00Z"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			action, next, err := DecideScheduleTick("0 * * * *", "UTC", tc.nextRun, now)
			if err != nil {
				t.Fatal(err)
			}
			if action != tc.wantAct {
				t.Fatalf("action = %s, want %s", action, tc.wantAct)
			}
			if next != tc.wantNext {
				t.Fatalf("next = %s, want %s", next, tc.wantNext)
			}
		})
	}
}
