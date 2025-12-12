package lsp

import (
    "bytes"
    "testing"
)

func TestStdioStream_ReadWriteClose(t *testing.T) {
    in := bytes.NewBufferString("abc")
    var out bytes.Buffer
    s := stdioStream{in: in, out: &out}
    n, err := s.Write([]byte("hi"))
    if err != nil || n != 2 {
        t.Fatalf("write err=%v n=%d", err, n)
    }
    if out.String() != "hi" {
        t.Fatalf("expected out hi, got %q", out.String())
    }
    buf := make([]byte, 3)
    rn, rerr := s.Read(buf)
    if rerr != nil || rn != 3 {
        t.Fatalf("read err=%v n=%d", rerr, rn)
    }
    if string(buf) != "abc" {
        t.Fatalf("expected read abc, got %q", string(buf))
    }
    if cerr := s.Close(); cerr != nil {
        t.Fatalf("close err=%v", cerr)
    }
}

