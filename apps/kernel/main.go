// apps/kernel/main.go
// Jupyter kernel entry point for Sruja Architecture Kernel

package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"time"

	"github.com/sruja-ai/sruja/pkg/kernel"
	"github.com/sruja-ai/sruja/pkg/kernel/jupyter"
)

func main() {
	connFile := flag.String("f", "", "Path to connection file (JSON)")
	logFile := flag.String("log", "", "Path to log file (optional, defaults to stderr)")
	flag.Parse()

	// Set up logging
	if *logFile != "" {
		f, err := os.OpenFile(*logFile, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to open log file: %v\n", err)
			os.Exit(1)
		}
		defer f.Close()
		log.SetOutput(f)
		log.SetFlags(log.LstdFlags | log.Lmicroseconds)
	} else {
		// Use stderr with timestamp
		log.SetFlags(log.LstdFlags | log.Lmicroseconds)
	}

	// Create kernel instance
	k, err := kernel.NewKernel()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to create kernel: %v\n", err)
		os.Exit(1)
	}

	// Generate session ID
	sessionID := fmt.Sprintf("sruja-%d", time.Now().Unix())

	// Create Jupyter server
	server := jupyter.NewServer(k, sessionID)

	// Set connection file if provided (and not the literal placeholder)
	if *connFile != "" && *connFile != "{connection_file}" {
		server.SetConnectionFile(*connFile)
	}

	// Start server (stdio mode if no connection file, ZeroMQ if connection file provided)
	// This will block until the kernel is shut down
	if err := server.Serve(); err != nil {
		fmt.Fprintf(os.Stderr, "Server error: %v\n", err)
		os.Exit(1)
	}

	// Should not reach here - server.Serve() blocks until shutdown
	fmt.Fprintf(os.Stderr, "Server exited unexpectedly\n")
	os.Exit(1)
}
