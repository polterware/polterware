package observability

import (
	"crypto/rand"
	"encoding/hex"
)

func NewCorrelationID() string {
	var b [16]byte
	_, _ = rand.Read(b[:])
	return hex.EncodeToString(b[:])
}
