package attachments

import (
	"net/netip"
	"net/url"
	"testing"
)

func TestSSRFGuardRejectsPrivateDestinations(t *testing.T) {
	for _, raw := range []string{"http://127.0.0.1/", "http://10.0.0.1/", "http://169.254.1.2/", "http://[::1]/", "http://localhost/"} {
		t.Run(raw, func(t *testing.T) {
			u, e := url.Parse(raw)
			if e != nil {
				t.Fatal(e)
			}
			if e = validatePublicURL(u); e == nil {
				t.Fatal("private destination accepted")
			}
		})
	}
}

func TestSSRFGuardRejectsPrivateResolvedAddress(t *testing.T) {
	if !blocked(netip.MustParseAddr("192.168.1.2")) {
		t.Fatal("private resolved address not blocked")
	}
}
