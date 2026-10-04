package store

import "strings"

// FTSQuery turns user text into a tokenizer-safe FTS5 query: each term is
// quoted, FTS operators are dropped, and the last term gets a prefix star.
// Text wrapped in double quotes is kept as a phrase, operators included.
func FTSQuery(user string) string {
	user = strings.TrimSpace(user)
	if len(user) >= 2 && strings.HasPrefix(user, `"`) && strings.HasSuffix(user, `"`) {
		return user
	}
	cleaned := strings.Map(func(r rune) rune {
		switch r {
		case '"', '*', ':', '^', '(', ')', '{', '}', '+', '-':
			return ' '
		default:
			return r
		}
	}, user)
	fields := strings.Fields(cleaned)
	if len(fields) == 0 {
		return ""
	}
	for i, field := range fields {
		fields[i] = `"` + strings.ReplaceAll(field, `"`, `""`) + `"`
	}
	fields[len(fields)-1] += "*"
	return strings.Join(fields, " ")
}
