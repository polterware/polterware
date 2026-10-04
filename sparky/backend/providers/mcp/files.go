package mcp

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

func checkAttachPath(path string) error {
	if strings.TrimSpace(path) == "" {
		return fmt.Errorf("file path is required")
	}
	if strings.Contains(path, "\x00") {
		return fmt.Errorf("file path is invalid")
	}
	cleaned := filepath.Clean(path)
	if strings.Contains(path, "..") || strings.Contains(cleaned, "..") {
		return fmt.Errorf("file path must not contain '..'")
	}
	info, err := os.Lstat(path)
	if err != nil {
		return fmt.Errorf("file path is not readable")
	}
	if info.Mode()&os.ModeSymlink != 0 {
		return fmt.Errorf("file path must be a regular file, not a symlink")
	}
	if !info.Mode().IsRegular() {
		return fmt.Errorf("file path must be a regular file")
	}
	if info.Size() > 50<<20 {
		return fmt.Errorf("file exceeds the 50 MB agent attach limit")
	}
	return nil
}
