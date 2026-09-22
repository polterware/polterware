package importer

import (
	"os"
	"path/filepath"
	"strings"
)

var rootExactNames = []string{
	"go.mod",
	"package.json",
	"AGENTS.md",
	"CLAUDE.md",
	"README.md",
	"Dockerfile",
	"docker-compose.yml",
	"docker-compose.yaml",
}

var openAPINames = []string{
	"openapi.yaml",
	"openapi.yml",
	"openapi.json",
	"swagger.yaml",
	"swagger.yml",
	"swagger.json",
}

var openAPIDirs = []string{
	"",
	"docs",
	"api",
	"openapi",
	"swagger",
}

var terraformDirs = []string{
	"",
	"terraform",
	"infra",
}

// Discover finds MVP source files under rootPath and parses each.
func Discover(rootPath string) (Result, error) {
	rootPath = filepath.Clean(rootPath)
	info, err := os.Stat(rootPath)
	if err != nil {
		return Result{}, err
	}
	if !info.IsDir() {
		return Result{}, os.ErrNotExist
	}

	seen := map[string]struct{}{}
	var paths []string

	add := func(rel string) {
		rel = filepath.ToSlash(rel)
		if rel == "" || strings.HasPrefix(rel, "../") {
			return
		}
		if _, ok := seen[rel]; ok {
			return
		}
		abs := filepath.Join(rootPath, filepath.FromSlash(rel))
		st, err := os.Stat(abs)
		if err != nil || st.IsDir() {
			return
		}
		seen[rel] = struct{}{}
		paths = append(paths, rel)
	}

	for _, name := range rootExactNames {
		add(name)
	}
	for _, dir := range openAPIDirs {
		for _, name := range openAPINames {
			if dir == "" {
				add(name)
			} else {
				add(filepath.ToSlash(filepath.Join(dir, name)))
			}
		}
	}
	for _, dir := range terraformDirs {
		var entries []os.DirEntry
		var err error
		if dir == "" {
			entries, err = os.ReadDir(rootPath)
		} else {
			entries, err = os.ReadDir(filepath.Join(rootPath, dir))
		}
		if err != nil {
			continue
		}
		for _, entry := range entries {
			if entry.IsDir() {
				continue
			}
			name := entry.Name()
			if !strings.HasSuffix(name, ".tf") {
				continue
			}
			if dir == "" {
				add(name)
			} else {
				add(filepath.ToSlash(filepath.Join(dir, name)))
			}
		}
	}

	out := Result{}
	for _, rel := range paths {
		abs := filepath.Join(rootPath, filepath.FromSlash(rel))
		data, err := os.ReadFile(abs)
		if err != nil {
			out.Skipped++
			out.Files = append(out.Files, FileResult{Path: rel, Skipped: true, Reason: err.Error()})
			continue
		}
		parsed := parseFile(rel, data)
		if parsed.Skipped {
			out.Skipped++
		}
		out.Files = append(out.Files, parsed)
	}
	return out, nil
}

func parseFile(rel string, data []byte) FileResult {
	base := filepath.Base(rel)
	lower := strings.ToLower(base)
	switch {
	case base == "go.mod":
		return parseGoMod(rel, data)
	case base == "package.json":
		return parsePackageJSON(rel, data)
	case base == "AGENTS.md" || base == "CLAUDE.md" || base == "README.md":
		return parseMarkdown(rel, data)
	case base == "Dockerfile":
		return parseDockerfile(rel, data)
	case lower == "docker-compose.yml" || lower == "docker-compose.yaml":
		return parseCompose(rel, data)
	case isOpenAPIName(lower):
		return parseOpenAPI(rel, data)
	case strings.HasSuffix(lower, ".tf"):
		return parseTerraform(rel, data)
	default:
		return FileResult{Path: rel, Skipped: true, Reason: "unsupported source"}
	}
}

func isOpenAPIName(lowerBase string) bool {
	for _, name := range openAPINames {
		if lowerBase == name {
			return true
		}
	}
	return false
}
