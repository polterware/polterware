package importer

import (
	"bufio"
	"bytes"
	"path/filepath"
	"strings"
)

func parseMarkdown(rel string, data []byte) FileResult {
	source := ImportSource(rel)
	base := filepath.Base(rel)
	evidence := DraftEvidence{
		Path:    rel,
		Summary: base + " project document",
		Source:  source,
	}
	facts := []DraftFact{{
		Key:       "present",
		Statement: "Project has " + base,
		Source:    source,
	}}
	scanner := bufio.NewScanner(bytes.NewReader(data))
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if strings.HasPrefix(line, "# ") {
			heading := strings.TrimSpace(strings.TrimPrefix(line, "# "))
			if heading != "" {
				facts = append(facts, DraftFact{
					Key:       "heading",
					Statement: base + " title is " + heading,
					Source:    source,
				})
			}
			break
		}
	}
	return FileResult{Path: rel, Evidence: evidence, Facts: facts, Title: base}
}
