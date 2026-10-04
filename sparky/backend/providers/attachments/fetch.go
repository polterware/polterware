package attachments

import (
	"context"
	"errors"
	"io"
	"net"
	"net/http"
	"net/netip"
	"net/url"
	"strings"
	"time"
)

const maxTitleBody = 1 << 20

// FetchTitle is an opt-in, bounded title fetch. Every initial and redirected host is DNS-checked.
func FetchTitle(ctx context.Context, raw string) (string, error) {
	transport := &http.Transport{DialContext: func(ctx context.Context, network, address string) (net.Conn, error) {
		host, port, err := net.SplitHostPort(address)
		if err != nil {
			return nil, err
		}
		ips, err := net.DefaultResolver.LookupNetIP(ctx, "ip", host)
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
	}}
	defer transport.CloseIdleConnections()
	client := &http.Client{Transport: transport, Timeout: 5 * time.Second, CheckRedirect: func(req *http.Request, via []*http.Request) error {
		if len(via) >= 5 {
			return errors.New("too many redirects")
		}
		if err := validatePublicURL(req.URL); err != nil {
			return err
		}
		return nil
	}}
	u, err := url.Parse(raw)
	if err != nil {
		return "", err
	}
	if err = validatePublicURL(u); err != nil {
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

func validatePublicURL(u *url.URL) error {
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
	ips, err := net.DefaultResolver.LookupNetIP(context.Background(), "ip", host)
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
	return !ip.IsValid() || ip.IsPrivate() || ip.IsLoopback() || ip.IsLinkLocalUnicast() || ip.IsLinkLocalMulticast() || ip.IsUnspecified() || ip.IsMulticast()
}
