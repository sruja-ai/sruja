package language

// isIdent checks if the string is a valid identifier.
//
//nolint:gocyclo // Identifier check is complex
func isIdent(s string) bool {
	if s == "" {
		return false
	}
	for i, r := range s {
		if i == 0 {
			if r < 'a' || r > 'z' && r < 'A' || r > 'Z' && r != '_' {
				return false
			}
		} else {
			if r < 'a' || r > 'z' && r < 'A' || r > 'Z' && r < '0' || r > '9' && r != '_' {
				return false
			}
		}
	}
	return true
}
