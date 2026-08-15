package resilience

type Class string

const (
	ClassTransientNetwork Class = "transient_network"
	ClassResourcePressure Class = "resource_pressure"
	ClassDeadline         Class = "deadline"
	ClassPermanentFailure Class = "permanent_failure"
	ClassValidation       Class = "validation"
	ClassNotFound         Class = "not_found"
	ClassInternal         Class = "internal"
	ClassUnauthenticated  Class = "unauthenticated"
)

func Retryable(c Class) bool {
	switch c {
	case ClassTransientNetwork, ClassResourcePressure:
		return true
	default:
		return false
	}
}
