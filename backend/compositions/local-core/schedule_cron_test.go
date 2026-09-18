package localcore

import (
	"testing"
	"time"
)

func TestParseScheduleCronRejectsInvalid(t *testing.T) {
	t.Parallel()
	if _, _, err := ParseScheduleCron("not a cron", "UTC"); err == nil {
		t.Fatal("expected invalid cron error")
	}
	if _, _, err := ParseScheduleCron("0 * * * *", "Not/AZone"); err == nil {
		t.Fatal("expected invalid timezone error")
	}
}

func TestNextRunAfterHourly(t *testing.T) {
	t.Parallel()
	sched, loc, err := ParseScheduleCron("0 * * * *", "UTC")
	if err != nil {
		t.Fatal(err)
	}
	from := time.Date(2026, 9, 18, 14, 30, 0, 0, time.UTC)
	next := NextRunAfter(sched, loc, from)
	want := time.Date(2026, 9, 18, 15, 0, 0, 0, time.UTC)
	if !next.Equal(want) {
		t.Fatalf("next = %v, want %v", next, want)
	}
}

func TestDecideScheduleTickEmptyAdvances(t *testing.T) {
	t.Parallel()
	now := time.Date(2026, 9, 18, 14, 30, 0, 0, time.UTC)
	action, next, err := DecideScheduleTick("0 * * * *", "UTC", "", now)
	if err != nil {
		t.Fatal(err)
	}
	if action != scheduleTickAdvance {
		t.Fatalf("action = %s, want advance", action)
	}
	if next != "2026-09-18T15:00:00Z" {
		t.Fatalf("next = %s", next)
	}
}

func TestDecideScheduleTickFutureWaits(t *testing.T) {
	t.Parallel()
	now := time.Date(2026, 9, 18, 14, 30, 0, 0, time.UTC)
	action, next, err := DecideScheduleTick("0 * * * *", "UTC", "2026-09-18T15:00:00Z", now)
	if err != nil {
		t.Fatal(err)
	}
	if action != scheduleTickWait {
		t.Fatalf("action = %s, want wait", action)
	}
	if next != "2026-09-18T15:00:00Z" {
		t.Fatalf("next = %s", next)
	}
}

func TestDecideScheduleTickNearDueFires(t *testing.T) {
	t.Parallel()
	now := time.Date(2026, 9, 18, 15, 0, 10, 0, time.UTC)
	action, next, err := DecideScheduleTick("0 * * * *", "UTC", "2026-09-18T15:00:00Z", now)
	if err != nil {
		t.Fatal(err)
	}
	if action != scheduleTickFire {
		t.Fatalf("action = %s, want fire", action)
	}
	if next != "2026-09-18T16:00:00Z" {
		t.Fatalf("next = %s", next)
	}
}

func TestDecideScheduleTickFarPastSkips(t *testing.T) {
	t.Parallel()
	now := time.Date(2026, 9, 18, 18, 0, 0, 0, time.UTC)
	action, next, err := DecideScheduleTick("0 * * * *", "UTC", "2026-09-18T15:00:00Z", now)
	if err != nil {
		t.Fatal(err)
	}
	if action != scheduleTickAdvance {
		t.Fatalf("action = %s, want advance (skip missed)", action)
	}
	if next != "2026-09-18T19:00:00Z" {
		t.Fatalf("next = %s", next)
	}
}
