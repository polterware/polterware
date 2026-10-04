package schedule

import (
	"html"
	"strings"
	"unicode"
	"unicode/utf8"
)

// ShapeBody produces the first 140 Unicode characters of plain text. It removes
// Markdown presentation, link destinations, HTML tags, and list/check markers,
// preserving readable labels and code contents. Scanning stops at the prefix.
func ShapeBody(note string) string {
	out := make([]rune, 0, 140)
	space, lineStart, fenced := false, true, false
	appendText := func(text string) {
		for _, r := range text {
			if len(out) == 140 {
				break
			}
			if unicode.IsSpace(r) {
				space = len(out) != 0
				continue
			}
			if space {
				out = append(out, ' ')
				space = false
			}
			if len(out) < 140 {
				out = append(out, r)
			}
		}
	}
	for i := 0; i < len(note) && len(out) < 140; {
		if strings.HasPrefix(note[i:], "```") || strings.HasPrefix(note[i:], "~~~") {
			fenced = !fenced
			if end := strings.IndexByte(note[i:], '\n'); end >= 0 {
				i += end + 1
			} else {
				break
			}
			lineStart, space = true, len(out) != 0
			continue
		}
		r, size := utf8.DecodeRuneInString(note[i:])
		if r == '\n' || r == '\r' {
			lineStart = true
		}
		if unicode.IsSpace(r) {
			space = len(out) != 0
			i += size
			continue
		}
		if !fenced {
			if lineStart && (r == '#' || r == '>' || r == '-' || r == '+' || r == '*') {
				i += size
				continue
			}
			if lineStart && r >= '0' && r <= '9' {
				j := i
				for j < len(note) && note[j] >= '0' && note[j] <= '9' {
					j++
				}
				if j+1 < len(note) && (note[j] == '.' || note[j] == ')') && note[j+1] == ' ' {
					i = j + 2
					continue
				}
			}
			if lineStart && (strings.HasPrefix(note[i:], "[ ] ") || strings.HasPrefix(note[i:], "[x] ") || strings.HasPrefix(note[i:], "[X] ")) {
				i += 4
				continue
			}
			if r == '<' {
				if end := strings.IndexByte(note[i:], '>'); end >= 0 {
					inside := note[i+1 : i+end]
					if strings.HasPrefix(inside, "https://") || strings.HasPrefix(inside, "http://") {
						appendText(inside)
						i += end + 1
						lineStart = false
						continue
					}
					i += end + 1
					continue
				}
			}
			if r == '!' && i+1 < len(note) && note[i+1] == '[' {
				i++
				continue
			}
			if r == '[' {
				i++
				continue
			}
			if r == ']' {
				i++
				if i < len(note) && note[i] == '(' {
					depth := 1
					i++
					for i < len(note) && depth > 0 {
						if note[i] == '(' {
							depth++
						}
						if note[i] == ')' {
							depth--
						}
						i++
					}
				} else if i < len(note) && note[i] == '[' {
					if end := strings.IndexByte(note[i:], ']'); end >= 0 {
						i += end + 1
					}
				}
				continue
			}
			if r == '*' || r == '_' || r == '`' || r == '~' {
				i += size
				continue
			}
			if r == '\\' && i+size < len(note) {
				i += size
				r, size = utf8.DecodeRuneInString(note[i:])
			}
			if r == '&' {
				end := strings.IndexByte(note[i:min(i+16, len(note))], ';')
				if end >= 0 {
					encoded := note[i : i+end+1]
					decoded := html.UnescapeString(encoded)
					if decoded != encoded {
						appendText(decoded)
						i += end + 1
						lineStart = false
						continue
					}
				}
			}
		}
		if space && len(out) < 140 {
			out = append(out, ' ')
			space = false
		}
		if len(out) < 140 {
			out = append(out, r)
		}
		lineStart = false
		i += size
	}
	return strings.TrimSpace(string(out))
}
