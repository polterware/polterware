package errors

import "testing"

func TestRetryable(t *testing.T) {
	if !MapRetryable(CodeResourceExhausted) {
		t.Fatal()
	}
	if MapRetryable(CodePermissionDenied) {
		t.Fatal()
	}
}
