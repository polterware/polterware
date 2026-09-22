package importer

import (
	"bufio"
	"bytes"
	"strings"
)

func parseDockerfile(rel string, data []byte) FileResult {
	source := ImportSource(rel)
	evidence := DraftEvidence{
		Path:    rel,
		Summary: "Dockerfile container definition",
		Source:  source,
	}
	var facts []DraftFact
	scanner := bufio.NewScanner(bytes.NewReader(data))
	seen := map[string]struct{}{}
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		upper := strings.ToUpper(line)
		if !strings.HasPrefix(upper, "FROM ") {
			continue
		}
		rest := strings.TrimSpace(line[4:])
		fields := strings.Fields(rest)
		if len(fields) == 0 {
			continue
		}
		image := fields[0]
		if strings.EqualFold(image, "scratch") {
			continue
		}
		if _, ok := seen[image]; ok {
			continue
		}
		seen[image] = struct{}{}
		facts = append(facts, DraftFact{
			Key:       "from:" + image,
			Statement: "Dockerfile uses image " + image,
			Source:    source,
		})
	}
	if len(facts) == 0 {
		facts = append(facts, DraftFact{
			Key:       "present",
			Statement: "Project has Dockerfile",
			Source:    source,
		})
	}
	return FileResult{Path: rel, Evidence: evidence, Facts: facts, Title: "Dockerfile"}
}
