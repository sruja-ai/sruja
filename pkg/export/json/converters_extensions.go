package json

import (
	"strconv"

	"github.com/sruja-ai/sruja/pkg/language"
)

func stringPtrToIntPtr(s *string) *int {
	if s == nil {
		return nil
	}
	val, err := strconv.Atoi(*s)
	if err != nil {
		return nil
	}
	return &val
}

// ============================================================================
// Parity Conversion Helpers
// ============================================================================

func convertSLO(slo *language.SLOBlock) *SLOJSON {
	if slo == nil {
		return nil
	}
	out := &SLOJSON{}
	if slo.Availability != nil {
		out.Availability = &SLOAvailabilityJSON{
			Target:  strVal(slo.Availability.Target),
			Window:  strVal(slo.Availability.Window),
			Current: slo.Availability.Current,
		}
	}
	if slo.Latency != nil {
		l := &SLOLatencyJSON{
			P95:    strVal(slo.Latency.P95),
			P99:    strVal(slo.Latency.P99),
			Window: strVal(slo.Latency.Window),
		}
		if slo.Latency.Current != nil {
			l.Current = &SLOCurrentJSON{
				P95: strVal(slo.Latency.Current.P95),
				P99: strVal(slo.Latency.Current.P99),
			}
		}
		out.Latency = l
	}
	if slo.ErrorRate != nil {
		out.ErrorRate = &SLOErrorRateJSON{
			Target:  strVal(slo.ErrorRate.Target),
			Window:  strVal(slo.ErrorRate.Window),
			Current: slo.ErrorRate.Current,
		}
	}
	if slo.Throughput != nil {
		out.Throughput = &SLOThroughputJSON{
			Target:  strVal(slo.Throughput.Target),
			Window:  strVal(slo.Throughput.Window),
			Current: slo.Throughput.Current,
		}
	}
	return out
}

func convertScale(scale *language.ScaleBlock) *ScaleJSON {
	if scale == nil {
		return nil
	}
	return &ScaleJSON{
		Min:    scale.Min,
		Max:    scale.Max,
		Metric: scale.Metric,
	}
}

func convertPolicies(policies []*language.Policy) []PolicyJSON {
	res := make([]PolicyJSON, 0, len(policies))
	for _, p := range policies {
		res = append(res, convertPolicy(p))
	}
	return res
}

func convertPolicy(p *language.Policy) PolicyJSON {
	pj := PolicyJSON{
		ID:          p.ID,
		Description: p.Description,
	}

	if p.Category != nil {
		pj.Category = *p.Category
	}
	if p.Enforcement != nil {
		pj.Enforcement = *p.Enforcement
	}
	if p.Body != nil {
		pj.Tags = p.Body.Tags
	}

	return pj
}

func convertConstraints(constraints []*language.ConstraintEntry) []ConstraintJSON {
	if len(constraints) == 0 {
		return nil
	}
	out := make([]ConstraintJSON, 0, len(constraints))
	for _, c := range constraints {
		out = append(out, ConstraintJSON{Key: c.Key, Value: c.Value})
	}
	return out
}

func convertConventions(conventions []*language.ConventionEntry) []ConventionJSON {
	if len(conventions) == 0 {
		return nil
	}
	out := make([]ConventionJSON, 0, len(conventions))
	for _, c := range conventions {
		out = append(out, ConventionJSON{Key: c.Key, Value: c.Value})
	}
	return out
}
