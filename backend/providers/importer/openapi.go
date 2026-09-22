package importer

import (
	"encoding/json"
	"regexp"
	"strings"
)

var (
	openAPITitleRE   = regexp.MustCompile(`(?m)^[ \t]*title:\s*["']?([^"'\n#]+)["']?`)
	openAPIVersionRE = regexp.MustCompile(`(?m)^[ \t]*version:\s*["']?([^"'\n#]+)["']?`)
)

func parseOpenAPI(rel string, data []byte) FileResult {
	source := ImportSource(rel)
	evidence := DraftEvidence{
		Path:    rel,
		Summary: "OpenAPI / Swagger contract",
		Source:  source,
	}
	title, version := extractOpenAPIInfo(data)
	var facts []DraftFact
	if title != "" {
		facts = append(facts, DraftFact{
			Key:       "title",
			Statement: "OpenAPI title is " + title,
			Source:    source,
		})
	}
	if version != "" {
		facts = append(facts, DraftFact{
			Key:       "version",
			Statement: "OpenAPI version is " + version,
			Source:    source,
		})
	}
	if len(facts) == 0 {
		facts = append(facts, DraftFact{
			Key:       "present",
			Statement: "Project has OpenAPI document " + rel,
			Source:    source,
		})
	}
	display := "OpenAPI"
	if title != "" {
		display = title
	}
	return FileResult{Path: rel, Evidence: evidence, Facts: facts, Title: display}
}

func extractOpenAPIInfo(data []byte) (title, version string) {
	trimmed := strings.TrimSpace(string(data))
	if strings.HasPrefix(trimmed, "{") {
		var payload struct {
			Info struct {
				Title   string `json:"title"`
				Version string `json:"version"`
			} `json:"info"`
		}
		if err := json.Unmarshal(data, &payload); err == nil {
			return strings.TrimSpace(payload.Info.Title), strings.TrimSpace(payload.Info.Version)
		}
	}
	if match := openAPITitleRE.FindStringSubmatch(trimmed); len(match) == 2 {
		title = strings.TrimSpace(match[1])
	}
	if match := openAPIVersionRE.FindStringSubmatch(trimmed); len(match) == 2 {
		version = strings.TrimSpace(match[1])
	}
	return title, version
}
