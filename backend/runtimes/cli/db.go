package main

import (
	"os"
	"path/filepath"
)

func defaultDB() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, "Library", "Application Support", "polterware", "local.db")
}
