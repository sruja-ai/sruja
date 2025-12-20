// pkg/language/ast_slo_test.go
package language

import (
	"testing"
)

func TestSLOBlockParsing(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		wantErr  bool
		checkSLO func(*SLOBlock) bool
	}{
		{
			name: "simple SLO with availability",
			input: `model {
				system API "API Service" {
					slo {
						availability {
							target "99.9%"
							window "30 days"
						}
					}
				}
			}`,
			wantErr: false,
			checkSLO: func(slo *SLOBlock) bool {
				return slo != nil &&
					slo.Availability != nil &&
					*slo.Availability.Target == "99.9%" &&
					*slo.Availability.Window == "30 days"
			},
		},
		{
			name: "SLO with all fields",
			input: `model {
				system API "API Service" {
					slo {
						availability {
							target "99.9%"
							window "30 days"
							current "99.95%"
						}
						latency {
							p95 "200ms"
							p99 "500ms"
							window "7 days"
							current {
								p95 "180ms"
								p99 "420ms"
							}
						}
						errorRate {
							target "0.1%"
							window "7 days"
							current "0.08%"
						}
						throughput {
							target "10000 req/s"
							window "peak hour"
							current "8500 req/s"
						}
					}
				}
			}`,
			wantErr: false,
			checkSLO: func(slo *SLOBlock) bool {
				return slo != nil &&
					slo.Availability != nil &&
					slo.Latency != nil &&
					slo.ErrorRate != nil &&
					slo.Throughput != nil &&
					slo.Latency.Current != nil &&
					*slo.Latency.Current.P95 == "180ms"
			},
		},
	}

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			program, diags, err := parser.Parse("test.sruja", tt.input)
			if tt.wantErr {
				if err == nil && len(diags) == 0 {
					t.Errorf("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Fatalf("Unexpected parse error: %v", err)
			}
			if len(diags) > 0 {
				t.Fatalf("Unexpected diagnostics: %v", diags)
			}

			if program == nil || program.Model == nil {
				t.Fatalf("Program or Model is nil")
			}

			// Find first system in Model and extract SLO
			var slo *SLOBlock
			for _, item := range program.Model.Items {
				if item.ElementDef != nil && item.ElementDef.GetKind() == "system" {
					// Extract SLO from LikeC4ElementDef body
					body := item.ElementDef.GetBody()
					if body != nil {
						for _, bodyItem := range body.Items {
							if bodyItem.SLO != nil {
								slo = bodyItem.SLO
								break
							}
						}
					}
				}
				if slo != nil {
					break
				}
			}
			if slo == nil {
				t.Errorf("SLO block not found in system")
				return
			}
			if !tt.checkSLO(slo) {
				t.Errorf("SLO block check failed")
			}
		})
	}
}
