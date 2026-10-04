// Package schedule is the recurrence rule, stored schedule, and reminder plan.
package schedule

import "time"

// Weekday bits. Bit 0 is Sunday.
const (
	WeekdaySunday uint8 = 1 << iota
	WeekdayMonday
	WeekdayTuesday
	WeekdayWednesday
	WeekdayThursday
	WeekdayFriday
	WeekdaySaturday
)

type Freq string

const (
	FreqMinutely Freq = "minutely"
	FreqHourly   Freq = "hourly"
	FreqDaily    Freq = "daily"
	FreqWeekly   Freq = "weekly"
	FreqMonthly  Freq = "monthly"
	FreqYearly   Freq = "yearly"
)

type EndType string

const (
	EndNever EndType = "never"
	EndUntil EndType = "until"
	EndCount EndType = "count"
)

const (
	CategoryReminder      = "reminder"
	CategoryReminderFocus = "reminder-focus"
	CategoryFocusPhase    = "focus-phase"
)

// End limits a series. Until is set only for EndUntil. Count is set only for EndCount.
type End struct {
	Type  EndType
	Until *time.Time
	Count int
}

// Rule is the pure recurrence value.
// An empty Freq is a one-shot. Every counts from 1.
// Weekly weekday masks use ISO weeks (Monday-start) from the anchor week.
type Rule struct {
	Anchor      time.Time
	TZ          *time.Location
	AllDay      bool
	Freq        Freq
	Every       int
	WeekdayMask uint8
	End         End
}

// Schedule is the schedules row, including the focus recipe.
type Schedule struct {
	MemoryID           string
	Active             bool
	FireAt             time.Time
	TZ                 string
	AllDay             bool
	Freq               Freq
	Every              int
	WeekdayMask        uint8
	EndType            EndType
	UntilAt            *time.Time
	OccurrenceCount    *int
	FocusEnabled       bool
	FocusWorkMin       *int
	FocusShortBreakMin *int
	FocusLongBreakMin  *int
	FocusCycles        *int
	FocusAutoContinue  bool
	UpdatedAt          time.Time
}

// Occurrence is one concrete instant in a range, with completion state.
type Occurrence struct {
	MemoryID     string
	Title        string
	OccurrenceAt time.Time
	FireAt       time.Time
	Completed    bool
	Overdue      bool
	AllDay       bool
	MindID       *string
	TZ           string
}

// DateComponents matches a native UNCalendarNotificationTrigger.
// Weekday uses the OS convention Sunday=1 … Saturday=7. Nil fields are wildcards.
type DateComponents struct {
	TZ      string
	Weekday *int
	Hour    *int
	Minute  *int
	Second  *int
}

func sameInt(a, b *int) bool {
	if a == nil || b == nil {
		return a == b
	}
	return *a == *b
}

func (a DateComponents) equal(b DateComponents) bool {
	return a.TZ == b.TZ && sameInt(a.Weekday, b.Weekday) && sameInt(a.Hour, b.Hour) &&
		sameInt(a.Minute, b.Minute) && sameInt(a.Second, b.Second)
}

func sameRepeat(a, b *DateComponents) bool {
	if a == nil || b == nil {
		return a == b
	}
	return a.equal(*b)
}

// ReminderItem is one notification. One-shots use rem:<memoryID>:<occurrenceUnixMs>
// or one:<id>. Native repeating items use rem:<memoryID>:repeat or rem:<memoryID>:wd<n>.
type ReminderItem struct {
	ID           string
	MemoryID     string
	OccurrenceAt time.Time
	FireAt       time.Time
	Title        string
	Body         string
	Category     string
	Repeat       *DateComponents
}

// PlanQuery is the planner window. Budget default 60. Horizon default 30 days.
type PlanQuery struct {
	Now     time.Time
	Budget  int
	Horizon time.Duration
}

// PlanSubject is one active memory the planner may remind.
type PlanSubject struct {
	MemoryID     string
	Title        string
	Note         string
	Rule         Rule
	CompletedAt  []time.Time
	FocusEnabled bool
	MindID       *string
	TagIDs       []string
	// Empty Status means active, preserving callers that supply active subjects.
	Status           string
	ScheduleInactive bool
}

// PendingOneOff is an extra notification (focus phase or snooze) merged into the plan.
type PendingOneOff struct {
	ID       string
	Kind     string
	MemoryID string
	FireAt   time.Time
	Title    string
	Body     string
}

// FocusRecipe is the per-memory pomodoro recipe stored on the schedule.
type FocusRecipe struct {
	Enabled       bool
	WorkMin       int
	ShortBreakMin int
	LongBreakMin  int
	Cycles        int
	AutoContinue  bool
}
