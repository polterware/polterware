package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"golang.org/x/term"
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
		if err := runTUI(*dbPath); err != nil {
			fail(err)
		}
		return
	}
	command := args[0]
	rest := args[1:]

	// Only `serve` opens the store (ADR 0007); every other command is a client
	// of the core and starts it when none is running.
	var err error
	switch command {
	case "serve":
		err = runServe(*dbPath, rest)
	case "mcp":
		err = runMCP(*dbPath)
	case "init":
		err = runInit(*dbPath, rest)
	case "scan":
		err = runScan(*dbPath, rest)
	case "context":
		err = runContext(*dbPath, rest)
	case "drift":
		err = runDrift(*dbPath, rest)
	case "apply":
		err = runApply(*dbPath, rest)
	case "lint":
		err = runLint(*dbPath, rest)
	case "format":
		err = runFormat(rest)
	case "query":
		err = runQuery(*dbPath, rest)
	case "hook":
		err = runHook(*dbPath, rest)
	case "help", "-h", "--help":
		usage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command %q\n", command)
		usage()
		os.Exit(2)
	}
	if err != nil {
		fail(err)
	}
}
