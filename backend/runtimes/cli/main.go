package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"golang.org/x/term"

	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func main() {
	dbPath := flag.String("db", defaultDB(), "sqlite path")
	flag.Usage = usage
	flag.Parse()
	if abs, err := filepath.Abs(*dbPath); err == nil {
		*dbPath = abs
	}

	args := flag.Args()
	if len(args) == 0 {
		if !term.IsTerminal(int(os.Stdin.Fd())) || !term.IsTerminal(int(os.Stdout.Fd())) {
			usage()
			os.Exit(2)
		}
		store, err := storepkg.New(*dbPath)
		if err != nil {
			fail(fmt.Errorf("db open: %w", err))
		}
		defer store.Close()
		if err := runTUI(store, *dbPath); err != nil {
			fail(err)
		}
		return
	}
	command := args[0]
	rest := args[1:]

	store, err := storepkg.New(*dbPath)
	if err != nil {
		fail(fmt.Errorf("db open: %w", err))
	}
	defer store.Close()

	switch command {
	case "init":
		if err := runInit(store, rest); err != nil {
			fail(err)
		}
	case "scan":
		if err := runScan(store, rest); err != nil {
			fail(err)
		}
	case "context":
		if err := runContext(store, rest); err != nil {
			fail(err)
		}
	case "drift":
		if err := runDrift(store, rest); err != nil {
			fail(err)
		}
	case "help", "-h", "--help":
		usage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command %q\n", command)
		usage()
		os.Exit(2)
	}
}
