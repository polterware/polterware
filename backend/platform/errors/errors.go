package errors

type Code string

const (
	CodeUnauthenticated  Code = "unauthenticated"
	CodePermissionDenied Code = "permission_denied"
	CodeInvalidArgument  Code = "invalid_argument"
	CodeNotFound         Code = "not_found"
	CodeResourceExhausted Code = "resource_exhausted"
	CodeDeadlineExceeded Code = "deadline_exceeded"
	CodeInternal         Code = "internal"
	CodeFailedPrecondition Code = "failed_precondition"
)

type Detail struct {
	Code          Code
	Message       string
	Retryable     bool
	CorrelationID string
}

func MapRetryable(c Code) bool {
	switch c {
	case CodeResourceExhausted, CodeInternal:
		return true
	default:
		return false
	}
}
