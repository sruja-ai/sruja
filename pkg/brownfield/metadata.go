package brownfield

import (
	"strconv"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

const (
	KeyInferred       = "sruja.inferred"
	KeyVerified       = "sruja.verified"
	KeyLocked         = "sruja.locked"
	KeyConfidence     = "sruja.confidence"
	KeyOrigin         = "sruja.origin"
	KeyExplanation    = "sruja.explanation"
	KeyBrownfieldMode = "sruja.brownfield_mode"
)

func metaToMap(meta []*language.MetaEntry) map[string]string {
	m := map[string]string{}
	for _, e := range meta {
		m[e.Key] = e.Value
	}
	return m
}

func IsInferred(meta []*language.MetaEntry) bool {
	m := metaToMap(meta)
	return strings.ToLower(m[KeyInferred]) == "true"
}

func IsVerified(meta []*language.MetaEntry) bool {
	m := metaToMap(meta)
	v := strings.ToLower(m[KeyVerified])
	if v == "" {
		return true
	}
	return v == "true"
}

func IsLocked(meta []*language.MetaEntry) bool {
	m := metaToMap(meta)
	return strings.ToLower(m[KeyLocked]) == "true"
}

func Confidence(meta []*language.MetaEntry) float64 {
	m := metaToMap(meta)
	if c, err := strconv.ParseFloat(m[KeyConfidence], 64); err == nil {
		return c
	}
	return 0.0
}

func Origin(meta []*language.MetaEntry) []string {
	m := metaToMap(meta)
	if m[KeyOrigin] == "" {
		return nil
	}
	parts := strings.Split(m[KeyOrigin], ";")
	for i := range parts {
		parts[i] = strings.TrimSpace(parts[i])
	}
	return parts
}

func Explanation(meta []*language.MetaEntry) string {
	m := metaToMap(meta)
	return m[KeyExplanation]
}

func BrownfieldMode(arch *language.Architecture) bool {
	if arch == nil {
		return false
	}
	m := metaToMap(arch.Metadata)
	return strings.ToLower(m[KeyBrownfieldMode]) == "true"
}
