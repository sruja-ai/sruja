package main

import (
	"context"
	"io"
	"os"

	"github.com/sourcegraph/jsonrpc2"
	"github.com/sruja-ai/sruja/pkg/lsp"
)

type stdioRW struct{}

func (stdioRW) Read(p []byte) (int, error)  { return os.Stdin.Read(p) }
func (stdioRW) Write(p []byte) (int, error) { return os.Stdout.Write(p) }
func (stdioRW) Close() error                { return nil }

func main() {
	var rwc io.ReadWriteCloser = stdioRW{}
	stream := jsonrpc2.NewBufferedStream(rwc, jsonrpc2.VSCodeObjectCodec{})
	conn := jsonrpc2.NewConn(context.Background(), stream, &lsp.Handler{})
	<-conn.DisconnectNotify()
}
