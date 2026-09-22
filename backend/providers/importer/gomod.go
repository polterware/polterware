package importer

import (
	"bufio"
	"bytes"
	"strings"
)

func parseGoMod(rel string, data []byte) FileResult {
	source := ImportSource(rel)
	evidence := DraftEvidence{
		Path:    rel,
		Summary: "go.mod module manifest",
		Source:  source,
	}
	var facts []DraftFact
	scanner := bufio.NewScanner(bytes.NewReader(data))
	inRequire := false
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "//") {
			continue
		}
		if strings.HasPrefix(line, "module ") {
			mod := strings.TrimSpace(strings.TrimPrefix(line, "module "))
			if mod != "" {
				facts = append(facts, DraftFact{
					Key:       "module",
					Statement: "Go module is " + mod,
					Source:    source,
				})
			}
			continue
		}
		if line == "require (" {
			inRequire = true
			continue
		}
		if inRequire {
			if line == ")" {
				inRequire = false
				continue
			}
			fields := strings.Fields(line)
			if len(fields) >= 1 && !strings.HasPrefix(fields[0], "//") {
				dep := fields[0]
				facts = append(facts, DraftFact{
					Key:       "require:" + dep,
					Statement: "Go module requires " + dep,
					Source:    source,
				})
			}
			continue
		}
		if strings.HasPrefix(line, "require ") {
			rest := strings.TrimSpace(strings.TrimPrefix(line, "require "))
			fields := strings.Fields(rest)
			if len(fields) >= 1 {
				dep := fields[0]
				facts = append(facts, DraftFact{
					Key:       "require:" + dep,
					Statement: "Go module requires " + dep,
					Source:    source,
				})
			}
		}
	}
	if len(facts) == 0 {
		facts = append(facts, DraftFact{
			Key:       "present",
			Statement: "Project has go.mod",
			Source:    source,
		})
	}
	return FileResult{Path: rel, Evidence: evidence, Facts: facts, Title: "go.mod"}
}
