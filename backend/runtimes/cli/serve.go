package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	localcore "github.com/polterware/polterware/backend/compositions/local-core"
	"github.com/polterware/polterware/backend/platform/runstate"
	"github.com/polterware/polterware/backend/providers/mcp"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// runServe runs the loopback Connect server the Desktop and TUI attach to.
func runServe(store *storepkg.Store, dbPath string, args []string) error {
	fs := flag.NewFlagSet("serve", flag.ContinueOnError)
	listenAddress := fs.String("listen", "127.0.0.1:4020", "loopback listen address")
	sessionToken := fs.String("token", os.Getenv("POLTERWARE_SESSION_TOKEN"), "session bearer token")
	if err := fs.Parse(args); err != nil {
		return err
	}

	if existing, ok := runstate.ReadCore(dbPath); ok && existing.Owner == "standalone" {
		fmt.Fprintf(os.Stderr, "polterware core already running for this database (pid %d, %s)\n", existing.PID, existing.Endpoint)
		os.Exit(3)
	}

	token := *sessionToken
	if token == "" {
		minted, err := localcore.MintToken()
		if err != nil {
			return fmt.Errorf("token: %w", err)
		}
		token = minted
	}
	ln, endpoint, err := localcore.Listen(*listenAddress)
	if err != nil {
		return fmt.Errorf("listen: %w", err)
	}
	fmt.Fprintf(os.Stderr, "polterware core listening on %s with session-token authentication\n", endpoint)

	core := localcore.NewCore(store, token)
	hs := localcore.ServeConnect(ln, core)

	owner := os.Getenv("POLTERWARE_CORE_OWNER")
	if owner == "" {
		owner = "standalone"
	}
	if err := runstate.WriteJSON(runstate.CorePath(dbPath), runstate.Core{
		PID:       os.Getpid(),
		Endpoint:  endpoint,
		Token:     token,
		DBPath:    dbPath,
		Owner:     owner,
		StartedAt: time.Now().UTC(),
	}); err != nil {
		fmt.Fprintf(os.Stderr, "run state: %v\n", err)
	}
	defer runstate.RemoveCore(dbPath, os.Getpid())

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = hs.Shutdown(ctx)
	return nil
}

// runMCP serves MCP over stdio for coding agents.
func runMCP(store *storepkg.Store, dbPath string) error {
	fmt.Fprintf(os.Stderr, "polterware mcp (experimental tools; not a product API contract)\n")
	return mcp.Run(store, dbPath)
}
