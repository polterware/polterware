package store

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

// OllamaProbe is the result of a best-effort local embedding call.
type OllamaProbe struct {
	Skipped bool
	Note    string
	Model   string
	Dims    int
	Latency time.Duration
}

// ProbeOllama calls a local Ollama if it is listening. A missing daemon is a skip, not an error.
// The embedded sentence is a fixed public pangram, never user data.
func ProbeOllama(ctx context.Context) (OllamaProbe, error) {
	ctx, cancel := context.WithTimeout(ctx, 8*time.Second)
	defer cancel()
	client := &http.Client{Timeout: 5 * time.Second}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, "http://127.0.0.1:11434/api/tags", nil)
	if err != nil {
		return OllamaProbe{}, fmt.Errorf("ollama tags request: %w", err)
	}
	res, err := client.Do(req)
	if err != nil {
		return OllamaProbe{Skipped: true, Note: "Ollama not reachable at 127.0.0.1:11434 (" + err.Error() + ")"}, nil
	}
	defer res.Body.Close()
	body, err := io.ReadAll(io.LimitReader(res.Body, 1<<20))
	if err != nil {
		return OllamaProbe{}, fmt.Errorf("read ollama tags: %w", err)
	}
	if res.StatusCode != http.StatusOK {
		return OllamaProbe{Skipped: true, Note: fmt.Sprintf("Ollama /api/tags returned %d", res.StatusCode)}, nil
	}
	var tags struct {
		Models []struct {
			Name string `json:"name"`
		} `json:"models"`
	}
	if err := json.Unmarshal(body, &tags); err != nil {
		return OllamaProbe{}, fmt.Errorf("decode ollama tags: %w", err)
	}
	model := ""
	for _, m := range tags.Models {
		if strings.Contains(strings.ToLower(m.Name), "embed") {
			model = m.Name
			break
		}
	}
	if model == "" {
		return OllamaProbe{Skipped: true, Note: fmt.Sprintf("Ollama is up with %d models, none has 'embed' in the name", len(tags.Models))}, nil
	}

	payload, err := json.Marshal(map[string]string{
		"model": model,
		"input": "The quick brown fox jumps over the lazy dog.",
	})
	if err != nil {
		return OllamaProbe{}, fmt.Errorf("encode embed request: %w", err)
	}
	start := time.Now()
	embedReq, err := http.NewRequestWithContext(ctx, http.MethodPost, "http://127.0.0.1:11434/api/embed", bytes.NewReader(payload))
	if err != nil {
		return OllamaProbe{}, fmt.Errorf("ollama embed request: %w", err)
	}
	embedReq.Header.Set("Content-Type", "application/json")
	embedRes, err := client.Do(embedReq)
	if err != nil {
		return OllamaProbe{Skipped: true, Note: "Ollama embed call failed: " + err.Error(), Model: model}, nil
	}
	defer embedRes.Body.Close()
	embedBody, err := io.ReadAll(io.LimitReader(embedRes.Body, 4<<20))
	if err != nil {
		return OllamaProbe{}, fmt.Errorf("read ollama embed: %w", err)
	}
	latency := time.Since(start)
	if embedRes.StatusCode != http.StatusOK {
		return OllamaProbe{Skipped: true, Model: model, Note: fmt.Sprintf("Ollama /api/embed returned %d: %s", embedRes.StatusCode, truncate(embedBody, 180))}, nil
	}
	var parsed struct {
		Embeddings [][]float64 `json:"embeddings"`
	}
	if err := json.Unmarshal(embedBody, &parsed); err != nil {
		return OllamaProbe{}, fmt.Errorf("decode ollama embed: %w", err)
	}
	if len(parsed.Embeddings) == 0 || len(parsed.Embeddings[0]) == 0 {
		return OllamaProbe{Skipped: true, Model: model, Note: "Ollama embed response had no vector"}, nil
	}
	return OllamaProbe{
		Model:   model,
		Dims:    len(parsed.Embeddings[0]),
		Latency: latency,
		Note:    fmt.Sprintf("embedded fixed pangram with %s (%d dims, %s)", model, len(parsed.Embeddings[0]), latency.Round(time.Millisecond)),
	}, nil
}

func truncate(b []byte, n int) string {
	s := string(b)
	if len(s) > n {
		return s[:n]
	}
	return s
}
