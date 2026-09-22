package ppm

import (
	"encoding/json"
	"strings"
)

// ParseLayoutNodes extracts component refs from architecture layout body_json.
// Supports formatVersion 1 (name) and legacy seed (label).
func ParseLayoutNodes(bodyJSON string) []LayoutNode {
	if strings.TrimSpace(bodyJSON) == "" {
		return nil
	}
	var doc struct {
		Nodes []map[string]any `json:"nodes"`
	}
	if err := json.Unmarshal([]byte(bodyJSON), &doc); err != nil {
		return nil
	}
	out := make([]LayoutNode, 0, len(doc.Nodes))
	for _, node := range doc.Nodes {
		id, _ := node["id"].(string)
		if id == "" {
			continue
		}
		title := stringField(node, "name")
		if title == "" {
			title = stringField(node, "label")
		}
		if title == "" {
			title = id
		}
		out = append(out, LayoutNode{ID: id, Title: title})
	}
	return out
}

func stringField(node map[string]any, key string) string {
	value, _ := node[key].(string)
	return strings.TrimSpace(value)
}
