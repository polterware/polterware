package handserver

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"io"
	"net/http"
	"sync"
)

const sessionHeader = "Mcp-Session-Id"

// Handler is the streamable HTTP server: POST a JSON-RPC message, GET an SSE
// stream for server notifications, DELETE to close the session.
type Handler struct {
	server   *Server
	mu       sync.Mutex
	sessions map[string]*session
}

// NewHTTP serves store over streamable HTTP.
func NewHTTP(srv *Server) *Handler {
	return &Handler{server: srv, sessions: map[string]*session{}}
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		h.post(w, r)
	case http.MethodGet:
		h.get(w, r)
	case http.MethodDelete:
		h.delete(w, r)
	default:
		w.Header().Set("Allow", "GET, POST, DELETE")
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
	}
}

func (h *Handler) post(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(io.LimitReader(r.Body, 4<<20))
	if err != nil {
		http.Error(w, "read body", http.StatusBadRequest)
		return
	}
	var probe struct {
		Method string `json:"method"`
	}
	_ = json.Unmarshal(body, &probe)

	var sess *session
	if probe.Method == "initialize" {
		sess = h.open()
	} else {
		sess = h.lookup(r.Header.Get(sessionHeader))
		if sess == nil {
			http.Error(w, "unknown MCP session: initialize again", http.StatusNotFound)
			return
		}
	}

	var notes [][]byte
	response, ok := h.server.handle(sess, body, func(msg []byte) {
		notes = append(notes, msg)
	})
	for _, note := range notes {
		sess.publisher.publish(note)
	}
	w.Header().Set(sessionHeader, sess.id)
	if !ok {
		w.WriteHeader(http.StatusAccepted)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	_, _ = w.Write(response)
}

func (h *Handler) get(w http.ResponseWriter, r *http.Request) {
	sess := h.lookup(r.Header.Get(sessionHeader))
	if sess == nil {
		http.Error(w, "unknown MCP session: initialize again", http.StatusNotFound)
		return
	}
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming unsupported", http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(": ready\n\n"))
	flusher.Flush()

	events, cancel := sess.publisher.subscribe()
	defer cancel()
	for {
		select {
		case <-r.Context().Done():
			return
		case msg, open := <-events:
			if !open {
				return
			}
			_, _ = w.Write([]byte("event: message\ndata: "))
			_, _ = w.Write(msg)
			_, _ = w.Write([]byte("\n\n"))
			flusher.Flush()
		}
	}
}

func (h *Handler) delete(w http.ResponseWriter, r *http.Request) {
	h.mu.Lock()
	defer h.mu.Unlock()
	id := r.Header.Get(sessionHeader)
	if sess := h.sessions[id]; sess != nil {
		sess.publisher.close()
		delete(h.sessions, id)
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *Handler) open() *session {
	h.mu.Lock()
	defer h.mu.Unlock()
	sess := &session{id: newID(), publisher: newPublisher()}
	h.sessions[sess.id] = sess
	return sess
}

func (h *Handler) lookup(id string) *session {
	h.mu.Lock()
	defer h.mu.Unlock()
	return h.sessions[id]
}

func newID() string {
	var buf [16]byte
	if _, err := rand.Read(buf[:]); err != nil {
		return "session"
	}
	return hex.EncodeToString(buf[:])
}

type publisher struct {
	mu     sync.Mutex
	subs   []chan []byte
	closed bool
}

func newPublisher() *publisher { return &publisher{} }

func (p *publisher) publish(msg []byte) {
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.closed {
		return
	}
	for _, ch := range p.subs {
		select {
		case ch <- append([]byte(nil), msg...):
		default:
		}
	}
}

func (p *publisher) subscribe() (<-chan []byte, func()) {
	ch := make(chan []byte, 8)
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.closed {
		close(ch)
		return ch, func() {}
	}
	p.subs = append(p.subs, ch)
	return ch, func() { p.remove(ch) }
}

func (p *publisher) remove(ch chan []byte) {
	p.mu.Lock()
	defer p.mu.Unlock()
	for i, sub := range p.subs {
		if sub == ch {
			p.subs = append(p.subs[:i], p.subs[i+1:]...)
			return
		}
	}
}

func (p *publisher) close() {
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.closed {
		return
	}
	p.closed = true
	for _, ch := range p.subs {
		close(ch)
	}
	p.subs = nil
}
