package attachments

import (
	"context"
	"errors"
	"io"
	"net"
	"net/http"
	"net/netip"
	"net/url"
	"strconv"
	"strings"
	"time"
)

const maxTitleBody = 1 << 20

type ipResolver interface {
	LookupNetIP(ctx context.Context, network, host string) ([]netip.Addr, error)
}

var lookupIPs ipResolver = net.DefaultResolver

func dialPublic(ctx context.Context, network, address string) (net.Conn, error) {
	host, port, err := net.SplitHostPort(address)
	if err != nil {
		return nil, err
	}
	ips, err := lookupIPs.LookupNetIP(ctx, "ip", host)
	if err != nil {
		return nil, err
	}
	if len(ips) == 0 {
		return nil, errors.New("host has no addresses")
	}
	for _, ip := range ips {
		if blocked(ip) {
			return nil, errors.New("private address is not allowed")
		}
	}
	var last error
	for _, ip := range ips {
		c, e := (&net.Dialer{}).DialContext(ctx, network, net.JoinHostPort(ip.String(), port))
		if e == nil {
			return c, nil
		}
		last = e
	}
	return nil, last
}

func checkPublicRedirect(req *http.Request, via []*http.Request) error {
	if len(via) >= 5 {
		return errors.New("too many redirects")
	}
	return validatePublicURL(req.Context(), req.URL)
}

// FetchTitle is an opt-in, bounded title fetch. Every initial and redirected host is DNS-checked.
func FetchTitle(ctx context.Context, raw string) (string, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	// net/http may detach the dial context from the request deadline. Keep
	// DNS and dialing under the same fetch budget explicitly.
	transport := &http.Transport{DialContext: func(_ context.Context, network, address string) (net.Conn, error) {
		return dialPublic(ctx, network, address)
	}}
	defer transport.CloseIdleConnections()
	client := &http.Client{Transport: transport, CheckRedirect: checkPublicRedirect}
	u, err := url.Parse(raw)
	if err != nil {
		return "", err
	}
	if err = validatePublicURL(ctx, u); err != nil {
		return "", err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, u.String(), nil)
	if err != nil {
		return "", err
	}
	req.Header.Set("User-Agent", "Sparky/1.0 (+local attachment title fetch)")
	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()
	data, err := io.ReadAll(io.LimitReader(resp.Body, maxTitleBody+1))
	if err != nil {
		return "", err
	}
	if len(data) > maxTitleBody {
		return "", errors.New("title response exceeds 1 MB")
	}
	text := string(data)
	lower := strings.ToLower(text)
	start := strings.Index(lower, "<title>")
	if start < 0 {
		return "", nil
	}
	start += len("<title>")
	end := strings.Index(lower[start:], "</title>")
	if end < 0 {
		return "", nil
	}
	return strings.TrimSpace(text[start : start+end]), nil
}

func validatePublicURL(ctx context.Context, u *url.URL) error {
	if err := ctx.Err(); err != nil {
		return err
	}
	if u == nil || (u.Scheme != "http" && u.Scheme != "https") || u.Hostname() == "" || u.User != nil {
		return errors.New("URL must be public http(s)")
	}
	host := u.Hostname()
	if ip, err := netip.ParseAddr(host); err == nil {
		if blocked(ip) {
			return errors.New("private address is not allowed")
		}
		return nil
	}
	// Reject legacy inet_aton numeric forms rather than letting DNS interpret
	// octal/hex/short addresses differently from the HTTP dialer.
	numeric := true
	for _, part := range strings.Split(strings.TrimSuffix(host, "."), ".") {
		if _, err := strconv.ParseUint(part, 0, 32); err != nil {
			numeric = false
			break
		}
	}
	if numeric {
		return errors.New("noncanonical numeric IP is not allowed")
	}
	ips, err := lookupIPs.LookupNetIP(ctx, "ip", host)
	if err != nil {
		return err
	}
	if len(ips) == 0 {
		return errors.New("host has no addresses")
	}
	for _, ip := range ips {
		if blocked(ip) {
			return errors.New("private address is not allowed")
		}
	}
	return nil
}
func blocked(ip netip.Addr) bool {
	ip = ip.Unmap()
	return !ip.IsValid() || ip.IsPrivate() || ip.IsLoopback() || ip.IsLinkLocalUnicast() || ip.IsLinkLocalMulticast() || ip.IsUnspecified() || ip.IsMulticast()
}
