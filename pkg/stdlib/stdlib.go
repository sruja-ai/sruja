package stdlib

import (
	"embed"
)

// FS contains the standard library files.
//
//go:embed *.sruja
var FS embed.FS
