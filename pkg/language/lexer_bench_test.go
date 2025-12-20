package language

import (
	"testing"
)

func BenchmarkLexer_NextToken(b *testing.B) {
	input := `
		specification {
			element system
			element container
			element component
			element person
		}
		model {
			User = person "User"
			System = system "System" {
				Web = container "Web"
				API = container "API"
				DB = container "DB"
			}
			User -> System.Web "Uses"
			System.Web -> System.API "Calls"
			System.API -> System.DB "Reads/Writes"
		}
	`
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		l := NewLexer(input)
		for {
			tok := l.NextToken()
			if tok.Type == TOKEN_EOF {
				break
			}
		}
	}
}
