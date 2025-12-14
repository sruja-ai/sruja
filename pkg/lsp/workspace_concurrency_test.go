package lsp

import (
	"sync"
	"testing"

	"github.com/sourcegraph/go-lsp"
)

func TestWorkspace_Concurrency_AddAndRead(t *testing.T) {
	w := NewWorkspace()
	var wg sync.WaitGroup
	n := 50
	wg.Add(n)
	for i := 0; i < n; i++ {
		i := i
		go func() {
			defer wg.Done()
			uri := lsp.DocumentURI("file:///doc" + string(rune('A'+i)) + ".sruja")
			text := "system S" + string(rune('A'+i))
			w.AddDocument(uri, text, 1)
			_ = w.GetDocument(uri)
			_ = w.AllDocuments()
		}()
	}
	wg.Wait()
	all := w.AllDocuments()
	if len(all) != n {
		t.Fatalf("expected %d documents, got %d", n, len(all))
	}
}
