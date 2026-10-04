package guard

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestGuard(t *testing.T) {
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("X-Test", "ok")
		w.WriteHeader(http.StatusNoContent)
	})
	g := &Guard{Token: "spike-token", Port: "47600"}
	h := g.Handler(next)

	ok := func(req *http.Request) *http.Response {
		t.Helper()
		rec := httptest.NewRecorder()
		h.ServeHTTP(rec, req)
		return rec.Result()
	}

	good := httptest.NewRequest(http.MethodPost, "http://127.0.0.1:47600/mcp", nil)
	good.Host = "127.0.0.1:47600"
	good.Header.Set("Authorization", "Bearer spike-token")
	if res := ok(good); res.StatusCode != http.StatusNoContent {
		t.Fatalf("good request status = %d", res.StatusCode)
	} else if res.Header.Get("Access-Control-Allow-Origin") != "" {
		t.Fatal("guard wrote a CORS header")
	}

	localhost := httptest.NewRequest(http.MethodPost, "http://localhost:47600/mcp", nil)
	localhost.Host = "localhost:47600"
	localhost.Header.Set("Authorization", "Bearer spike-token")
	localhost.Header.Set("Origin", "http://127.0.0.1:47600")
	if res := ok(localhost); res.StatusCode != http.StatusNoContent {
		t.Fatalf("loopback origin status = %d", res.StatusCode)
	}

	cases := []struct {
		name   string
		host   string
		origin string
		auth   string
		want   int
	}{
		{name: "missing token", host: "127.0.0.1:47600", auth: "", want: http.StatusUnauthorized},
		{name: "wrong token", host: "127.0.0.1:47600", auth: "Bearer other-token", want: http.StatusUnauthorized},
		{name: "bad host", host: "evil.example:47600", auth: "Bearer spike-token", want: http.StatusForbidden},
		{name: "wrong port", host: "127.0.0.1:80", auth: "Bearer spike-token", want: http.StatusForbidden},
		{name: "bad origin", host: "127.0.0.1:47600", origin: "https://evil.example", auth: "Bearer spike-token", want: http.StatusForbidden},
		{name: "null origin", host: "127.0.0.1:47600", origin: "null", auth: "Bearer spike-token", want: http.StatusForbidden},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodPost, "http://127.0.0.1:47600/mcp", nil)
			req.Host = tc.host
			if tc.auth != "" {
				req.Header.Set("Authorization", tc.auth)
			}
			if tc.origin != "" {
				req.Header.Set("Origin", tc.origin)
			}
			if res := ok(req); res.StatusCode != tc.want {
				t.Fatalf("status = %d, want %d", res.StatusCode, tc.want)
			}
		})
	}
}
