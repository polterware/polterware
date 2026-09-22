package importer

import (
	"regexp"
	"strings"
)

var composeServiceRE = regexp.MustCompile(`(?m)^[ \t]{2}([A-Za-z0-9_.-]+):\s*(?:#.*)?$`)

func parseCompose(rel string, data []byte) FileResult {
	source := ImportSource(rel)
	evidence := DraftEvidence{
		Path:    rel,
		Summary: "Docker Compose service definition",
		Source:  source,
	}
	text := string(data)
	servicesIdx := strings.Index(text, "\nservices:")
	if servicesIdx < 0 && strings.HasPrefix(strings.TrimSpace(text), "services:") {
		servicesIdx = 0
	}
	var facts []DraftFact
	if servicesIdx >= 0 {
		section := text[servicesIdx:]
		endMarkers := []string{"\nnetworks:", "\nvolumes:", "\nconfigs:", "\nsecrets:", "\nx-"}
		end := len(section)
		for _, marker := range endMarkers {
			if i := strings.Index(section, marker); i > 0 && i < end {
				end = i
			}
		}
		section = section[:end]
		matches := composeServiceRE.FindAllStringSubmatch(section, -1)
		seen := map[string]struct{}{}
		for _, match := range matches {
			name := match[1]
			if name == "services" {
				continue
			}
			if _, ok := seen[name]; ok {
				continue
			}
			seen[name] = struct{}{}
			facts = append(facts, DraftFact{
				Key:       "service:" + name,
				Statement: "Compose service " + name,
				Source:    source,
			})
		}
	}
	if len(facts) == 0 {
		facts = append(facts, DraftFact{
			Key:       "present",
			Statement: "Project has Docker Compose file " + rel,
			Source:    source,
		})
	}
	return FileResult{Path: rel, Evidence: evidence, Facts: facts, Title: "docker-compose"}
}
