// Package guard is the loopback gate in front of both spike HTTP servers.
// It enforces the bearer token plus the Host and Origin checks from
// plans/sparky-next/01-architecture.md. It never writes CORS headers.
package guard

import (
	"context"
	"crypto/subtle"
	"errors"
	"fmt"
	"net"
	"net/http"
	"strconv"
	"strings"
	"time"
)

// Guard rejects requests that are not the local MCP client.
type Guard struct {
	Token string
	// Port is the listener port, compared to the Host header. Set it before
	// the first request. An empty Port fails closed.
	Port string
}

// Handler wraps next with authentication and DNS-rebinding checks.
func (g *Guard) Handler(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !bearerOK(r.Header.Get("Authorization"), g.Token) {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		if !hostOK(r.Host, g.Port) {
			http.Error(w, "forbidden host", http.StatusForbidden)
			return
		}
		if origin := r.Header.Get("Origin"); origin != "" && !originOK(origin) {
			http.Error(w, "forbidden origin", http.StatusForbidden)
			return
		}
		next.ServeHTTP(w, r)
	})
}

// Serve listens on a loopback address and serves the MCP handler at /mcp.
// It returns when ctx is cancelled.
func Serve(ctx context.Context, addr, token string, mcpHandler http.Handler) error {
	ln, err := net.Listen("tcp", addr)
	if err != nil {
		return err
	}
	tcp, ok := ln.Addr().(*net.TCPAddr)
	if !ok || tcp.IP == nil || !tcp.IP.IsLoopback() {
		_ = ln.Close()
		return fmt.Errorf("mcp listen address %s is not loopback", addr)
	}
	g := &Guard{Token: token, Port: strconv.Itoa(tcp.Port)}
	mux := http.NewServeMux()
	mux.Handle("/mcp", mcpHandler)
	server := &http.Server{
		Handler:           g.Handler(mux),
		ReadHeaderTimeout: 5 * time.Second,
	}
	errc := make(chan error, 1)
	go func() { errc <- server.Serve(ln) }()
	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer cancel()
		_ = server.Shutdown(shutdownCtx)
		return nil
	case err := <-errc:
		if errors.Is(err, http.ErrServerClosed) {
			return nil
		}
		return err
	}
}

func bearerOK(header, token string) bool {
	const prefix = "Bearer "
	if len(header) < len(prefix) || !strings.EqualFold(header[:len(prefix)], prefix) {
		return false
	}
	got := header[len(prefix):]
	if len(got) != len(token) || token == "" {
		return false
	}
	return subtle.ConstantTimeCompare([]byte(got), []byte(token)) == 1
}

func hostOK(host, port string) bool {
	if port == "" {
		return false
	}
	name, gotPort, err := net.SplitHostPort(host)
	if err != nil {
		return false
	}
	if gotPort != port {
		return false
	}
	return name == "127.0.0.1" || name == "localhost"
}

func originOK(origin string) bool {
	// Allow only http(s) origins whose host is loopback. A browser on another
	// site cannot satisfy this, which blocks CSRF and DNS rebinding.
	var rest string
	switch {
	case strings.HasPrefix(origin, "https://"):
		rest = strings.TrimPrefix(origin, "https://")
	case strings.HasPrefix(origin, "http://"):
		rest = strings.TrimPrefix(origin, "http://")
	default:
		return false
	}
	rest = strings.Split(rest, "/")[0]
	name, _, err := net.SplitHostPort(rest)
	if err != nil {
		name = rest
	}
	return name == "127.0.0.1" || name == "localhost" || name == "[::1]" || name == "::1"
}
