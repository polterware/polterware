// Throwaway core for the W0-NOTIFIER spike. It serves Plan, Complete, Snooze,
// Ack and a long-poll Watch on loopback. It is not the Sparky core.
package main

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"sync"
	"time"
)

type reminder struct {
	ID           string          `json:"id"`
	MemoryID     string          `json:"memoryId"`
	FireAt       string          `json:"fireAt"`
	Title        string          `json:"title"`
	Body         string          `json:"body"`
	Category     string          `json:"category"`
	Repeat       json.RawMessage `json:"repeat,omitempty"`
	OccurrenceAt string          `json:"occurrenceAt,omitempty"`
}

type event struct {
	At     string         `json:"at"`
	Kind   string         `json:"kind"`
	Fields map[string]any `json:"fields"`
}

type server struct {
	mu       sync.Mutex
	items    []reminder
	events   []event
	lastAck  map[string]any
	waiters  []chan struct{}
	eventsFD *os.File
	planPath string
}

func main() {
	data := flag.String("data", "", "spike data directory")
	addr := flag.String("addr", "127.0.0.1:0", "loopback listen address")
	flag.Parse()
	if *data == "" {
		log.Fatal("-data is required")
	}
	runDir := filepath.Join(*data, "run")
	if err := os.MkdirAll(runDir, 0o755); err != nil {
		log.Fatal(err)
	}
	eventsPath := filepath.Join(runDir, "events.jsonl")
	eventsFile, err := os.OpenFile(eventsPath, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0o600)
	if err != nil {
		log.Fatal(err)
	}
	defer eventsFile.Close()

	token, err := randomToken()
	if err != nil {
		log.Fatal(err)
	}
	listener, err := net.Listen("tcp", *addr)
	if err != nil {
		log.Fatal(err)
	}
	endpoint := "http://" + listener.Addr().String()
	core := map[string]any{
		"pid":       os.Getpid(),
		"endpoint":  endpoint,
		"token":     token,
		"build":     "w0-notifier-spike",
		"dbPath":    "",
		"startedAt": time.Now().UTC().Format(time.RFC3339Nano),
	}
	raw, err := json.MarshalIndent(core, "", "  ")
	if err != nil {
		log.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(runDir, "core.json"), append(raw, '\n'), 0o600); err != nil {
		log.Fatal(err)
	}

	srv := &server{eventsFD: eventsFile, planPath: filepath.Join(runDir, "plan.json")}
	if raw, err := os.ReadFile(srv.planPath); err == nil {
		if err := json.Unmarshal(raw, &srv.items); err != nil {
			log.Fatal(err)
		}
	}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", srv.authed(token, srv.health))
	mux.HandleFunc("GET /plan", srv.authed(token, srv.getPlan))
	mux.HandleFunc("POST /plan", srv.authed(token, srv.postPlan))
	mux.HandleFunc("POST /complete", srv.authed(token, srv.postEvent("complete")))
	mux.HandleFunc("POST /snooze", srv.authed(token, srv.postEvent("snooze")))
	mux.HandleFunc("POST /ack", srv.authed(token, srv.postAck))
	mux.HandleFunc("GET /events", srv.authed(token, srv.getEvents))
	mux.HandleFunc("GET /meta", srv.authed(token, srv.getMeta))
	mux.HandleFunc("GET /watch", srv.authed(token, srv.watch))

	fmt.Fprintf(os.Stderr, "stub core listening %s\n", endpoint)
	if err := http.Serve(listener, mux); err != nil {
		log.Fatal(err)
	}
}

func loopbackHost(host string) bool {
	name, _, err := net.SplitHostPort(host)
	if err != nil {
		name = host
	}
	return name == "127.0.0.1" || name == "localhost" || name == "::1"
}

func randomToken() (string, error) {
	buf := make([]byte, 32)
	if _, err := rand.Read(buf); err != nil {
		return "", err
	}
	return hex.EncodeToString(buf), nil
}

func (s *server) authed(token string, next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if !loopbackHost(r.Host) {
			http.Error(w, "bad host", http.StatusBadRequest)
			return
		}
		got := r.Header.Get("Authorization")
		want := "Bearer " + token
		if subtle.ConstantTimeCompare([]byte(got), []byte(want)) != 1 {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		next(w, r)
	}
}

func (s *server) health(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]any{"ok": true, "build": "w0-notifier-spike"})
}

func (s *server) getPlan(w http.ResponseWriter, _ *http.Request) {
	s.mu.Lock()
	items := make([]reminder, 0, len(s.items))
	for _, item := range s.items {
		fire, err := time.Parse(time.RFC3339Nano, item.FireAt)
		repeating := len(item.Repeat) > 0 && string(item.Repeat) != "null"
		if repeating || (err == nil && fire.After(time.Now())) {
			items = append(items, item)
		}
	}
	s.mu.Unlock()
	if items == nil {
		items = []reminder{}
	}
	writeJSON(w, http.StatusOK, map[string]any{"items": items})
}

func (s *server) postPlan(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Items []reminder `json:"items"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	s.mu.Lock()
	s.items = body.Items
	if err := s.persistPlan(); err != nil {
		s.mu.Unlock()
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	s.mu.Unlock()
	s.wake()
	writeJSON(w, http.StatusOK, map[string]any{"ok": true, "count": len(body.Items)})
}

func (s *server) postEvent(kind string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var fields map[string]any
		if err := json.NewDecoder(r.Body).Decode(&fields); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		s.record(kind, fields)
		s.mu.Lock()
		memory, _ := fields["memoryId"].(string)
		items := make([]reminder, 0, len(s.items))
		for _, item := range s.items {
			if item.MemoryID != memory {
				items = append(items, item)
			}
		}
		if kind == "snooze" {
			id, _ := fields["occurrenceId"].(string)
			items = append(items, reminder{ID: id + ":snooze", MemoryID: memory, FireAt: time.Now().UTC().Add(10 * time.Minute).Format("2006-01-02T15:04:05.000Z"), Title: "W0 snoozed", Body: "Synthetic native notification spike", Category: "reminder"})
		}
		s.items = items
		err := s.persistPlan()
		s.mu.Unlock()
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		writeJSON(w, http.StatusOK, map[string]any{"ok": true, "applied": true})
	}
}

// Called with s.mu held; the persisted plan survives the spike's SIGKILL tests.
func (s *server) persistPlan() error {
	raw, err := json.Marshal(s.items)
	if err != nil {
		return err
	}
	return os.WriteFile(s.planPath, raw, 0o600)
}

func (s *server) postAck(w http.ResponseWriter, r *http.Request) {
	var fields map[string]any
	if err := json.NewDecoder(r.Body).Decode(&fields); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	s.mu.Lock()
	s.lastAck = fields
	s.mu.Unlock()
	s.record("ack", fields)
	writeJSON(w, http.StatusOK, map[string]any{"ok": true})
}

func (s *server) getEvents(w http.ResponseWriter, _ *http.Request) {
	s.mu.Lock()
	events := append([]event(nil), s.events...)
	s.mu.Unlock()
	if events == nil {
		events = []event{}
	}
	writeJSON(w, http.StatusOK, map[string]any{"events": events})
}

func (s *server) getMeta(w http.ResponseWriter, _ *http.Request) {
	s.mu.Lock()
	defer s.mu.Unlock()
	writeJSON(w, http.StatusOK, map[string]any{
		"lastAck":    s.lastAck,
		"eventCount": len(s.events),
		"planCount":  len(s.items),
	})
}

func (s *server) watch(w http.ResponseWriter, r *http.Request) {
	timeout := 25 * time.Second
	if raw := r.URL.Query().Get("timeout"); raw != "" {
		if parsed, err := time.ParseDuration(raw + "s"); err == nil && parsed > 0 && parsed <= 60*time.Second {
			timeout = parsed
		}
	}
	ch := make(chan struct{})
	s.mu.Lock()
	s.waiters = append(s.waiters, ch)
	s.mu.Unlock()
	timer := time.NewTimer(timeout)
	defer timer.Stop()
	select {
	case <-ch:
		writeJSON(w, http.StatusOK, map[string]any{"woke": true})
	case <-timer.C:
		writeJSON(w, http.StatusOK, map[string]any{"woke": false})
	case <-r.Context().Done():
	}
}

func (s *server) record(kind string, fields map[string]any) {
	item := event{At: time.Now().UTC().Format(time.RFC3339Nano), Kind: kind, Fields: fields}
	raw, err := json.Marshal(item)
	if err != nil {
		return
	}
	s.mu.Lock()
	s.events = append(s.events, item)
	_, _ = s.eventsFD.Write(append(raw, '\n'))
	s.mu.Unlock()
	s.wake()
}

func (s *server) wake() {
	s.mu.Lock()
	waiters := s.waiters
	s.waiters = nil
	s.mu.Unlock()
	for _, ch := range waiters {
		close(ch)
	}
}

func writeJSON(w http.ResponseWriter, code int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(payload)
}
