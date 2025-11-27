package main

import (
	"log"

	"github.com/sruja-ai/sruja/mcp"
)

func main() {
	if err := mcp.Start(":8090"); err != nil {
		log.Fatal(err)
	}
}
