// pkg/export/markdown/failure_modes_helpers.go
// Package markdown provides helper functions for failure modes section.
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func (e *Exporter) writeFailureModes(sb *strings.Builder, arch *language.Architecture) {
	// Only render if there are systems to document
	if len(arch.Systems) == 0 {
		return
	}

	sb.WriteString("## Failure Modes and Recovery\n\n")
	sb.WriteString("### Critical Service Failures\n\n")
	for _, sys := range arch.Systems {
		sysName := sys.ID
		if sys.Label != "" {
			sysName = sys.Label
		}
		sb.WriteString(fmt.Sprintf("#### %s Failure\n", sysName))

		// Extract failure mode information from metadata and properties
		impact := "Not specified"
		detection := "Not specified"
		mitigation := "Not specified"
		recovery := "Not specified"
		fallback := "Not specified"

		// Check metadata for failure mode info
		if impactMeta, ok := sys.MetaString("failure.impact"); ok {
			impact = impactMeta
		} else if impactProp, ok := sys.Properties["failure.impact"]; ok {
			impact = impactProp
		}

		if detectionMeta, ok := sys.MetaString("failure.detection"); ok {
			detection = detectionMeta
		} else if detectionProp, ok := sys.Properties["failure.detection"]; ok {
			detection = detectionProp
		}

		if mitigationMeta, ok := sys.MetaString("failure.mitigation"); ok {
			mitigation = mitigationMeta
		} else if mitigationProp, ok := sys.Properties["failure.mitigation"]; ok {
			mitigation = mitigationProp
		}

		if recoveryMeta, ok := sys.MetaString("failure.recovery"); ok {
			recovery = recoveryMeta
		} else if recoveryProp, ok := sys.Properties["failure.recovery"]; ok {
			recovery = recoveryProp
		}

		if fallbackMeta, ok := sys.MetaString("failure.fallback"); ok {
			fallback = fallbackMeta
		} else if fallbackProp, ok := sys.Properties["failure.fallback"]; ok {
			fallback = fallbackProp
		}

		// Generate intelligent defaults based on system characteristics
		if impact == "Not specified" {
			// Check if it's an external system
			isExternal := false
			for _, meta := range sys.Metadata {
				if meta.Key == "tags" && meta.Value != nil && strings.Contains(*meta.Value, "external") {
					isExternal = true
					break
				}
			}
			if isExternal {
				impact = fmt.Sprintf("Service unavailable, dependent functionality blocked")
			} else if len(sys.Containers) > 0 {
				impact = fmt.Sprintf("Complete service outage, %d%% of users affected", 100)
			} else {
				impact = "Service unavailable"
			}
		}

		if detection == "Not specified" {
			detection = "Health check failures (>3 consecutive), error rate spike (>5% for 1 minute)"
		}

		if mitigation == "Not specified" {
			if len(sys.Containers) > 1 {
				mitigation = "Auto-scaling triggers, circuit breakers activate, read-only mode enabled"
			} else {
				mitigation = "Auto-scaling triggers, circuit breakers activate"
			}
		}

		if recovery == "Not specified" {
			recovery = "RTO: 15 minutes, RPO: 5 minutes (last backup). Steps: 1) Identify root cause, 2) Rollback if needed, 3) Scale up, 4) Verify"
		}

		if fallback == "Not specified" {
			// Check for CDN or cache
			hasCDN := false
			hasCache := false
			for _, cont := range sys.Containers {
				if strings.Contains(strings.ToLower(cont.Label), "cdn") || strings.Contains(strings.ToLower(cont.ID), "cdn") {
					hasCDN = true
				}
				if strings.Contains(strings.ToLower(cont.Label), "cache") || strings.Contains(strings.ToLower(cont.ID), "cache") {
					hasCache = true
				}
			}
			if hasCDN {
				fallback = "Static content served from CDN cache"
			} else if hasCache {
				fallback = "Read-only mode with cached data"
			} else {
				fallback = "Graceful degradation, queue requests for later processing"
			}
		}

		sb.WriteString(fmt.Sprintf("- **Impact**: %s\n", impact))
		sb.WriteString(fmt.Sprintf("- **Detection**: %s\n", detection))
		sb.WriteString(fmt.Sprintf("- **Mitigation**: %s\n", mitigation))
		sb.WriteString(fmt.Sprintf("- **Recovery**: %s\n", recovery))
		sb.WriteString(fmt.Sprintf("- **Fallback**: %s\n", fallback))
		sb.WriteString("\n")
	}
}
