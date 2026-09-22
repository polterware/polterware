package mcp

import "encoding/json"

func marshalPretty(value any) (string, error) {
	body, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		return "", err
	}
	return string(body), nil
}

func stringFrom(value any) string {
	text, _ := value.(string)
	return text
}

func intFrom(value any, fallback int) int {
	switch typed := value.(type) {
	case float64:
		return int(typed)
	case int:
		return typed
	case int32:
		return int(typed)
	case json.Number:
		n, err := typed.Int64()
		if err == nil {
			return int(n)
		}
	}
	return fallback
}
