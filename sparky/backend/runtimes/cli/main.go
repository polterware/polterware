package main

import (
	"fmt"
	"os"
)

const version = "0.0.0-dev"

func main() {
	args := os.Args[1:]
	if len(args) == 0 {
		fmt.Printf("sparky %s\n", version)
		return
	}

	switch args[0] {
	case "serve", "mcp", "status", "doctor":
		fmt.Printf("sparky %s: %s (stub)\n", version, args[0])
	default:
		fmt.Fprintf(os.Stderr, "sparky %s: unknown command %q\n", version, args[0])
		os.Exit(2)
	}
}
