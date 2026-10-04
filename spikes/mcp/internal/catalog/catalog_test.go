package catalog

import (
	"testing"
	"unicode/utf8"
)

func TestBudgets(t *testing.T) {
	tools := []string{DescTime, DescList, DescCreate, DescToday, DescPromptCapture}
	for _, desc := range tools {
		if n := utf8.RuneCountInString(desc); n > 300 {
			t.Errorf("description is %d runes, budget is 300: %q", n, desc)
		}
	}
	params := []string{DescLimit, DescCursor, DescItems, DescTitle, DescNote}
	for _, desc := range params {
		if n := utf8.RuneCountInString(desc); n > 120 {
			t.Errorf("parameter description is %d runes, budget is 120: %q", n, desc)
		}
	}
	if n := utf8.RuneCountInString(PromptCaptureText); n > 1024 {
		t.Errorf("prompt is %d runes, budget is 1024", n)
	}
}
