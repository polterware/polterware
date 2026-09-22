package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	localcore "github.com/polterware/polterware/backend/compositions/local-core"
	"github.com/polterware/polterware/backend/providers/mcp"
	sqlstore "github.com/polterware/polterware/backend/providers/sqlite"
)

func main() {
	dbPath := flag.String("db", defaultDB(), "sqlite path")
	listenAddress := flag.String("listen", "127.0.0.1:4020", "loopback listen address")
	sessionToken := flag.String("token", os.Getenv("POLTERWARE_SESSION_TOKEN"), "session bearer token")
	flag.Parse()

	args := flag.Args()
	mode := "serve"
	if len(args) > 0 {
		mode = args[0]
	}

	store, err := sqlstore.New(*dbPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "db open: %v\n", err)
		os.Exit(1)
	}
	defer store.Close()

	switch mode {
	case "mcp":
		fmt.Fprintf(os.Stderr, "polterware-core mcp (experimental tools; not a product API contract)\n")
		if err := mcp.Run(store); err != nil {
			fmt.Fprintf(os.Stderr, "mcp: %v\n", err)
			os.Exit(1)
		}
		return
	case "serve", "":
		// default Connect server below
	default:
		fmt.Fprintf(os.Stderr, "unknown command %q (use serve or mcp)\n", mode)
		os.Exit(2)
	}

	token := *sessionToken
	if token == "" {
		token, err = localcore.MintToken()
		if err != nil {
			fmt.Fprintf(os.Stderr, "token: %v\n", err)
			os.Exit(1)
		}
	}
	ln, endpoint, err := localcore.Listen(*listenAddress)
	if err != nil {
		fmt.Fprintf(os.Stderr, "listen: %v\n", err)
		os.Exit(1)
	}
	fmt.Fprintf(os.Stderr, "polterware-core listening on %s with session-token authentication\n", endpoint)

	core := localcore.NewCore(store, token)
	core.StartScheduleRunner()
	defer core.StopScheduleRunner()
	hs := localcore.ServeConnect(ln, core)

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
	core.StopScheduleRunner()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = hs.Shutdown(ctx)
}

func defaultDB() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, "Library", "Application Support", "polterware", "local.db")
}
