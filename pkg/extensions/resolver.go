package extensions

import (
	"fmt"
	"strings"
)

// PackageRef represents a parsed Git package reference
type PackageRef struct {
	Host    string // e.g., "github.com"
	Owner   string // e.g., "myorg"
	Repo    string // e.g., "sruja-aws-rules"
	Version string // e.g., "v1.2.0" or commit hash
	Path    string // Optional subpath within repo
}

// ParsePackageRef parses a Git URL reference
// Formats supported:
//   - github.com/owner/repo@version
//   - github.com/owner/repo/subpath@version
func ParsePackageRef(ref string) (*PackageRef, error) {
	// Split on @
	parts := strings.Split(ref, "@")
	if len(parts) != 2 {
		return nil, fmt.Errorf("invalid package reference '%s': must be in format 'host/owner/repo@version'", ref)
	}

	urlPart := parts[0]
	version := parts[1]

	// Split URL part
	urlParts := strings.Split(urlPart, "/")
	if len(urlParts) < 3 {
		return nil, fmt.Errorf("invalid package reference '%s': must have at least host/owner/repo", ref)
	}

	pkg := &PackageRef{
		Host:    urlParts[0],
		Owner:   urlParts[1],
		Repo:    urlParts[2],
		Version: version,
	}

	// If there are more parts, they're the subpath
	if len(urlParts) > 3 {
		pkg.Path = strings.Join(urlParts[3:], "/")
	}

	return pkg, nil
}

// GitURL returns the full Git clone URL
func (p *PackageRef) GitURL() string {
	return fmt.Sprintf("https://%s/%s/%s.git", p.Host, p.Owner, p.Repo)
}

// String returns the canonical package reference string
func (p *PackageRef) String() string {
	base := fmt.Sprintf("%s/%s/%s", p.Host, p.Owner, p.Repo)
	if p.Path != "" {
		base = fmt.Sprintf("%s/%s", base, p.Path)
	}
	return fmt.Sprintf("%s@%s", base, p.Version)
}
