package resilience

import "testing"

func TestRetryClass(t *testing.T) {
	if !Retryable(ClassTransientNetwork) || Retryable(ClassPermanentFailure) {
		t.Fatal()
	}
}
