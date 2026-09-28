// Package ids generates stable record ids that can be created offline on any
// machine without collisions.
package ids

import (
	"crypto/rand"
	"time"
)

const crockford = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"

// NewULID returns a 26-character ULID: 48-bit millisecond timestamp followed by
// 80 random bits, Crockford base32 encoded. Ids sort by creation time.
func NewULID() string {
	return ulidAt(time.Now())
}

func ulidAt(now time.Time) string {
	var raw [16]byte
	ms := uint64(now.UnixMilli())
	for i := 5; i >= 0; i-- {
		raw[i] = byte(ms)
		ms >>= 8
	}
	_, _ = rand.Read(raw[6:])

	// 128 bits → 26 base32 characters (the first character carries 3 bits).
	var out [26]byte
	var acc uint64
	bits := 2 // leading padding so 130 bits split evenly into 5-bit groups
	pos := 0
	for _, b := range raw {
		acc = acc<<8 | uint64(b)
		bits += 8
		for bits >= 5 {
			bits -= 5
			out[pos] = crockford[(acc>>uint(bits))&0x1f]
			pos++
		}
	}
	return string(out[:])
}
