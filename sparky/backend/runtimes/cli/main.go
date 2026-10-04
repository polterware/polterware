package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"github.com/polterware/polterware/sparky/backend/platform/runstate"
	"io"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
)

const version = "0.0.0-dev"

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM, syscall.SIGHUP)
	defer stop()
	os.Exit(run(ctx, os.Args[1:], os.Stdout, os.Stderr))
}

func run(ctx context.Context, args []string, out, stderr io.Writer) int {
	fs := flag.NewFlagSet("sparky", flag.ContinueOnError)
	fs.SetOutput(stderr)
	defaultPath, err := defaultDB()
	if err != nil {
		fmt.Fprintln(stderr, err)
		return 1
	}
	db := fs.String("db", defaultPath, "SQLite database path")
	fs.Usage = func() {
		fmt.Fprintln(stderr, "Usage: sparky [-db path] <serve|mcp|status|doctor> [options]")
		fs.PrintDefaults()
	}
	if err := fs.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return 0
		}
		return 2
	}
	args = fs.Args()
	if len(args) == 0 {
		fmt.Fprintf(out, "sparky %s\n", version)
		return 0
	}
	path, err := filepath.Abs(*db)
	if err != nil {
		fmt.Fprintln(stderr, err)
		return 1
	}
	switch args[0] {
	case "serve":
		err = runServe(ctx, path, args[1:], stderr)
	case "mcp":
		err = runMCP(ctx, path, args[1:], os.Stdin, out, stderr)
	case "status":
		err = runStatus(ctx, path, args[1:], out, stderr)
	case "doctor":
		err = runDoctor(ctx, path, args[1:], out, stderr)
	default:
		fmt.Fprintf(stderr, "sparky %s: unknown command %q\n", version, args[0])
		return 2
	}
	if err != nil {
		fmt.Fprintf(stderr, "sparky: %v\n", err)
		var locked runstate.LockedError
		if errors.As(err, &locked) {
			return 3
		}
		return 1
	}
	return 0
}
func defaultDB() (string, error) {
	if data := os.Getenv("SPARKY_DATA_DIR"); data != "" {
		return filepath.Join(data, "sparky.db"), nil
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("find data directory: %w", err)
	}
	return filepath.Join(home, "Library", "Application Support", "Sparky", "sparky.db"), nil
}
