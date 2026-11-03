package main

import (
	"os"
	"path/filepath"
)

// defaultDB is the database shared with the Desktop: <user config dir>/polterware/local.db
// (Electron's appData on macOS, Linux, and Windows).
func defaultDB() string {
	dir, err := os.UserConfigDir()
	if err != nil {
		home, _ := os.UserHomeDir()
		dir = filepath.Join(home, ".config")
	}
	return filepath.Join(dir, "polterware", "local.db")
}
