package localcore

import (
	"fmt"
	"time"

	"github.com/robfig/cron/v3"
)

const schedulePollInterval = 15 * time.Second

// scheduleMissLag is how far past nextRunAt we still treat as an on-time fire.
// Larger lag means the schedule was missed while core was offline — skip without firing.
var scheduleMissLag = schedulePollInterval * 2

var cronParser = cron.NewParser(cron.Minute | cron.Hour | cron.Dom | cron.Month | cron.Dow)

// ParseScheduleCron parses a 5-field cron expression in the given IANA timezone.
func ParseScheduleCron(expression, timezone string) (cron.Schedule, *time.Location, error) {
	if timezone == "" {
		timezone = "UTC"
	}
	loc, err := time.LoadLocation(timezone)
	if err != nil {
		return nil, nil, fmt.Errorf("invalid timezone %q: %w", timezone, err)
	}
	sched, err := cronParser.Parse(expression)
	if err != nil {
		return nil, nil, fmt.Errorf("invalid cron expression %q: %w", expression, err)
	}
	return sched, loc, nil
}

// FormatScheduleInstant formats a time as RFC3339 UTC for schedule storage.
func FormatScheduleInstant(t time.Time) string {
	return t.UTC().Format(time.RFC3339)
}

// ParseScheduleInstant parses an RFC3339 (or RFC3339Nano) instant.
func ParseScheduleInstant(value string) (time.Time, error) {
	if value == "" {
		return time.Time{}, fmt.Errorf("empty schedule instant")
	}
	if t, err := time.Parse(time.RFC3339Nano, value); err == nil {
		return t.UTC(), nil
	}
	t, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return time.Time{}, err
	}
	return t.UTC(), nil
}

// NextRunAfter returns the next cron occurrence after from, evaluated in loc.
func NextRunAfter(sched cron.Schedule, loc *time.Location, from time.Time) time.Time {
	if loc == nil {
		loc = time.UTC
	}
	return sched.Next(from.In(loc)).UTC()
}

// NextRunAtString computes the next run instant string for a schedule expression.
func NextRunAtString(expression, timezone string, from time.Time) (string, error) {
	sched, loc, err := ParseScheduleCron(expression, timezone)
	if err != nil {
		return "", err
	}
	return FormatScheduleInstant(NextRunAfter(sched, loc, from)), nil
}

type scheduleTickAction string

const (
	scheduleTickWait   scheduleTickAction = "wait"
	scheduleTickAdvance scheduleTickAction = "advance" // skip missed / empty — no fire
	scheduleTickFire   scheduleTickAction = "fire"
)

// DecideScheduleTick applies skip-missed policy for one schedule at now.
func DecideScheduleTick(expression, timezone, nextRunAt string, now time.Time) (action scheduleTickAction, newNextRunAt string, err error) {
	sched, loc, err := ParseScheduleCron(expression, timezone)
	if err != nil {
		return "", "", err
	}

	nextFuture := FormatScheduleInstant(NextRunAfter(sched, loc, now))

	if nextRunAt == "" {
		return scheduleTickAdvance, nextFuture, nil
	}
	dueAt, parseErr := ParseScheduleInstant(nextRunAt)
	if parseErr != nil {
		return scheduleTickAdvance, nextFuture, nil
	}

	if dueAt.After(now) {
		return scheduleTickWait, nextRunAt, nil
	}

	lag := now.Sub(dueAt)
	if lag > scheduleMissLag {
		return scheduleTickAdvance, nextFuture, nil
	}
	return scheduleTickFire, nextFuture, nil
}
