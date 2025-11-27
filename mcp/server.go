package mcp

import (
	"encoding/json"
	"fmt"
	"io/fs"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/sruja-ai/sruja/pkg/approval"
	"github.com/sruja-ai/sruja/pkg/brownfield"
	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
	"github.com/sruja-ai/sruja/pkg/policy"
	"github.com/sruja-ai/sruja/pkg/query"
	"github.com/sruja-ai/sruja/pkg/review"
)

type Server struct{}

func Start(addr string) error {
	s := &Server{}
	mux := http.NewServeMux()
	mux.HandleFunc("/load", s.handleLoad)
	mux.HandleFunc("/explain", s.handleExplain)
	mux.HandleFunc("/query", s.handleQuery)
	mux.HandleFunc("/validate", s.handleValidate)
	mux.HandleFunc("/diagram", s.handleDiagram)
	mux.HandleFunc("/diff", s.handleDiff)
	mux.HandleFunc("/modify", s.handleModify)
	mux.HandleFunc("/resolve-code-location", s.handleResolveCodeLocation)
	mux.HandleFunc("/check-code-change-impact", s.handleCheckCodeChangeImpact)
	mux.HandleFunc("/suggest-refactor", s.handleSuggestRefactor)
	mux.HandleFunc("/relevant-adrs", s.handleRelevantADRs)
	mux.HandleFunc("/record-change", s.handleRecordChange)
	mux.HandleFunc("/get-change-history", s.handleGetChangeHistory)
	mux.HandleFunc("/search-code", s.handleSearchCode)
	mux.HandleFunc("/generate-hint", s.handleGenerateHint)
	mux.HandleFunc("/check-boundary", s.handleCheckBoundary)
	mux.HandleFunc("/review-architecture", s.handleReviewArchitecture)
	mux.HandleFunc("/evaluate-policy", s.handleEvaluatePolicy)
	mux.HandleFunc("/list-entities", s.handleListEntities)
	mux.HandleFunc("/list-events", s.handleListEvents)
	mux.HandleFunc("/validate-event", s.handleValidateEvent)
	mux.HandleFunc("/validate-approval-policy", s.handleValidateApprovalPolicy)
	mux.HandleFunc("/validate-all", s.handleValidateAllPolicies)
	return http.ListenAndServe(addr, mux)
}

func parseProgram(path string) (*language.Program, error) {
	p, err := language.NewParser()
	if err != nil {
		return nil, err
	}
	return p.ParseFile(path)
}

func (s *Server) handleLoad(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path string `json:"path"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	tr := compiler.NewTransformer()
	m, err := tr.Transform(prog)
	if err != nil {
		writeErr(w, err)
		return
	}
	resp := map[string]any{"architecture": m.Architecture}
	writeJSON(w, resp)
}

func (s *Server) handleExplain(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path string `json:"path"`
		ID   string `json:"id"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	ex := dx.NewExplainer(prog)
	out, err := ex.ExplainElement(body.ID)
	if err != nil {
		writeErr(w, err)
		return
	}
	writeJSON(w, out)
}

func (s *Server) handleQuery(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path string `json:"path"`
		Q    string `json:"q"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	eng := query.NewEngine(prog)
	res, err := eng.Execute(body.Q)
	if err != nil {
		writeErr(w, err)
		return
	}
	writeJSON(w, res)
}

func (s *Server) handleValidate(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, map[string]any{"errors": []any{}})
}

func (s *Server) handleDiagram(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path string `json:"path"`
		Type string `json:"type"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	format := "mermaid"
	if strings.Contains(strings.ToLower(body.Type), "d2") {
		format = "d2"
	}
	out, err := compiler.DefaultRegistry.Compile(format, prog)
	if err != nil {
		writeErr(w, err)
		return
	}
	writeJSON(w, map[string]any{"format": format, "data": out})
}

func (s *Server) handleDiff(w http.ResponseWriter, r *http.Request) {
	type req struct {
		A string `json:"a"`
		B string `json:"b"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	pa, err := parseProgram(body.A)
	if err != nil {
		writeErr(w, err)
		return
	}
	pb, err := parseProgram(body.B)
	if err != nil {
		writeErr(w, err)
		return
	}
	ta := compiler.NewTransformer()
	tb := compiler.NewTransformer()
	ma, err := ta.Transform(pa)
	if err != nil {
		writeErr(w, err)
		return
	}
	mb, err := tb.Transform(pb)
	if err != nil {
		writeErr(w, err)
		return
	}
	added, removed := diffElements(ma.Architecture.Elements, mb.Architecture.Elements)
	writeJSON(w, map[string]any{"elementsAdded": added, "elementsRemoved": removed})
}

func diffElements(a, b []model.Element) (added []string, removed []string) {
	am := map[string]bool{}
	bm := map[string]bool{}
	for _, e := range a {
		am[e.ID] = true
	}
	for _, e := range b {
		bm[e.ID] = true
	}
	for id := range bm {
		if !am[id] {
			added = append(added, id)
		}
	}
	for id := range am {
		if !bm[id] {
			removed = append(removed, id)
		}
	}
	return
}

func (s *Server) handleModify(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, map[string]any{"status": "not_implemented"})
}

// --- Architecture → Code Mapping ---
func (s *Server) handleResolveCodeLocation(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path    string `json:"path"`
		Element string `json:"element"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	paths := resolvePathsForElement(prog, body.Element)
	writeJSON(w, map[string]any{"paths": paths})
}

func resolvePathsForElement(p *language.Program, id string) []string {
	var out []string
	arch := p.Architecture
	if arch == nil {
		return out
	}
	addFromMeta := func(ms []*language.MetaEntry) {
		for _, m := range ms {
			if m.Key == "code.path" && m.Value != "" {
				out = append(out, m.Value)
			}
		}
	}
	for _, s := range arch.Systems {
		if s.ID == id {
			addFromMeta(s.Metadata)
		}
		for _, c := range s.Containers {
			if c.ID == id {
				addFromMeta(c.Metadata)
			}
			for _, comp := range c.Components {
				if comp.ID == id {
					addFromMeta(comp.Metadata)
				}
			}
		}
		for _, comp := range s.Components {
			if comp.ID == id {
				addFromMeta(comp.Metadata)
			}
		}
	}
	return uniqueStrings(out)
}

// --- Code Change Impact ---
func (s *Server) handleCheckCodeChangeImpact(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path   string `json:"path"`
		File   string `json:"file"`
		Change string `json:"change"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	elem := elementForFile(prog, body.File)
	target := targetFromChange(body.Change)
	allowed := relationExists(prog, elem, target)
	resp := map[string]any{"status": "ok"}
	if !allowed {
		resp = map[string]any{
			"status":         "warning",
			"message":        elem + " is not allowed to depend directly on " + target,
			"allowedImports": []string{target + ".PublicClient"},
			"suggestedFix":   "Use approved client or adapter per architecture relations",
		}
	}
	writeJSON(w, resp)
}

func elementForFile(p *language.Program, file string) string {
	arch := p.Architecture
	if arch == nil {
		return ""
	}
	match := func(ms []*language.MetaEntry, f string) bool {
		for _, m := range ms {
			if m.Key == "code.path" && strings.HasPrefix(f, m.Value) {
				return true
			}
		}
		return false
	}
	for _, s := range arch.Systems {
		if match(s.Metadata, file) {
			return s.ID
		}
		for _, c := range s.Containers {
			if match(c.Metadata, file) {
				return c.ID
			}
			for _, comp := range c.Components {
				if match(comp.Metadata, file) {
					return comp.ID
				}
			}
		}
		for _, comp := range s.Components {
			if match(comp.Metadata, file) {
				return comp.ID
			}
		}
	}
	return ""
}

func targetFromChange(change string) string {
	// naive extraction: last token after space
	parts := strings.Fields(change)
	if len(parts) > 0 {
		return parts[len(parts)-1]
	}
	return change
}

func relationExists(p *language.Program, from string, to string) bool {
	if from == "" || to == "" {
		return false
	}
	arch := p.Architecture
	for _, r := range arch.Relations {
		if r.From == from && r.To == to {
			return true
		}
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			if r.From == from && r.To == to {
				return true
			}
		}
	}
	return false
}

// --- Refactor Guidance ---
func (s *Server) handleSuggestRefactor(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path    string `json:"path"`
		Element string `json:"element"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	out := suggestRefactor(prog, body.Element)
	writeJSON(w, out)
}

func suggestRefactor(p *language.Program, id string) map[string]any {
	arch := p.Architecture
	outRel := 0
	for _, r := range arch.Relations {
		if r.From == id {
			outRel++
		}
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			if r.From == id {
				outRel++
			}
		}
	}
	if outRel > 10 {
		return map[string]any{
			"improvements": []map[string]any{{
				"issue":         id + " has high coupling (" + fmt.Sprintf("%d", outRel) + " outbound relations)",
				"suggestion":    "Create a worker module for background tasks",
				"codeLocations": resolvePathsForElement(p, id),
			}},
		}
	}
	return map[string]any{"improvements": []any{}}
}

// --- ADR-aware Guidance ---
func (s *Server) handleRelevantADRs(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path    string `json:"path"`
		Element string `json:"element"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	var adrs []map[string]any
	for _, adr := range prog.Architecture.ADRs {
		adrs = append(adrs, map[string]any{
			"id": adr.ID, "title": adr.Title, "status": "accepted", "suggestionForCode": "Adhere to decision in relevant modules",
		})
	}
	writeJSON(w, map[string]any{"adrs": adrs})
}

// --- Traceability ---
func (s *Server) handleRecordChange(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Element string   `json:"element"`
		Type    string   `json:"type"`
		Reason  string   `json:"reason"`
		Files   []string `json:"files"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	storeTrace(body.Element, body.Type, body.Reason, body.Files)
	writeJSON(w, map[string]any{"status": "ok"})
}

func (s *Server) handleGetChangeHistory(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Element string `json:"element"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	hist := loadTrace(body.Element)
	writeJSON(w, map[string]any{"history": hist})
}

// --- Architecture-aware File Search ---
func (s *Server) handleSearchCode(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path    string `json:"path"`
		Element string `json:"element"`
		Query   string `json:"query"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	paths := resolvePathsForElement(prog, body.Element)
	results := searchInPaths(paths, body.Query)
	writeJSON(w, map[string]any{"results": results})
}

// --- Architecture-driven Hints ---
func (s *Server) handleGenerateHint(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path            string `json:"path"`
		Task            string `json:"task"`
		Element         string `json:"element"`
		DesiredBehavior string `json:"desiredBehavior"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	files := []string{}
	for _, p := range resolvePathsForElement(prog, body.Element) {
		files = append(files, p+"/handlers/invoice.go", p+"/routes.go")
	}
	writeJSON(w, map[string]any{
		"owner":         body.Element,
		"filesToModify": uniqueStrings(files),
		"ruleNotes":     []string{"Follow ADRs and repository pattern", "Authenticate endpoints"},
	})
}

// --- Boundary Enforcement ---
func (s *Server) handleCheckBoundary(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path     string `json:"path"`
		FromFile string `json:"fromFile"`
		ToFile   string `json:"toFile"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	fromElem := elementForFile(prog, body.FromFile)
	toElem := elementForFile(prog, body.ToFile)
	allowed := relationExists(prog, fromElem, toElem)
	if allowed {
		writeJSON(w, map[string]any{"allowed": true})
		return
	}
	writeJSON(w, map[string]any{"allowed": false, "reason": "No relation defined between elements", "allowedTargets": []string{toElem}})
}

// --- helpers ---
func uniqueStrings(xs []string) []string {
	m := map[string]bool{}
	var out []string
	for _, x := range xs {
		if !m[x] {
			m[x] = true
			out = append(out, x)
		}
	}
	return out
}

func writeJSON(w http.ResponseWriter, v any) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(v)
}

func writeErr(w http.ResponseWriter, err error) {
	w.WriteHeader(http.StatusBadRequest)
	writeJSON(w, map[string]string{"error": err.Error()})
}

// trace store
type traceEntry struct {
	Element string   `json:"element"`
	Type    string   `json:"type"`
	Reason  string   `json:"reason"`
	Files   []string `json:"files"`
}

func traceFile() string { return ".sruja/trace.json" }

func ensureTraceDir() { os.MkdirAll(".sruja", 0755) }

func storeTrace(element, typ, reason string, files []string) {
	ensureTraceDir()
	var entries []traceEntry
	if data, err := os.ReadFile(traceFile()); err == nil {
		json.Unmarshal(data, &entries)
	}
	entries = append(entries, traceEntry{Element: element, Type: typ, Reason: reason, Files: files})
	data, _ := json.MarshalIndent(entries, "", "  ")
	os.WriteFile(traceFile(), data, 0644)
}

func loadTrace(element string) []traceEntry {
	var entries []traceEntry
	if data, err := os.ReadFile(traceFile()); err == nil {
		json.Unmarshal(data, &entries)
	}
	if element == "" {
		return entries
	}
	var out []traceEntry
	for _, e := range entries {
		if e.Element == element {
			out = append(out, e)
		}
	}
	return out
}

func searchInPaths(paths []string, q string) []string {
	var results []string
	visit := func(root string) {
		filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				return nil
			}
			if d.IsDir() {
				name := d.Name()
				if name == ".git" || name == "node_modules" || strings.HasPrefix(name, ".") {
					return filepath.SkipDir
				}
				return nil
			}
			// skip binaries
			if strings.HasSuffix(path, ".png") || strings.HasSuffix(path, ".svg") || strings.HasSuffix(path, ".pdf") {
				return nil
			}
			data, err := os.ReadFile(path)
			if err == nil && strings.Contains(strings.ToLower(string(data)), strings.ToLower(q)) {
				results = append(results, path)
			}
			return nil
		})
	}
	for _, p := range paths {
		visit(p)
	}
	return uniqueStrings(results)
}

func (s *Server) handleReviewArchitecture(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path  string `json:"path"`
		Rules string `json:"rules"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	rp, err := review.BuildRuleParser()
	if err != nil {
		writeErr(w, err)
		return
	}
	rs, err := rp.ParseString("rules.srujarule", body.Rules)
	if err != nil {
		writeErr(w, err)
		return
	}
	rules := review.ToRules(rs)
	diags := review.Evaluate(prog.Architecture, rules)
	writeJSON(w, map[string]any{"diagnostics": diags})
}

func (s *Server) handleEvaluatePolicy(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path   string `json:"path"`
		Policy string `json:"policy"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	pp, err := policy.BuildPolicyParser()
	if err != nil {
		writeErr(w, err)
		return
	}
	pf, err := pp.ParseString("policy.srujapolicy", body.Policy)
	if err != nil {
		writeErr(w, err)
		return
	}
	rules := map[string]review.Rule{}
	res := []policy.Evaluation{}
	for _, p := range pf.Policies {
		ev := policy.EvaluatePolicy(prog.Architecture, rules, p)
		res = append(res, ev)
	}
	writeJSON(w, map[string]any{"results": res})
}

func (s *Server) handleListEntities(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path string `json:"path"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	var list []map[string]any
	arch := prog.Architecture
	for _, e := range collectEntities(arch) {
		// Entity doesn't have Metadata field yet, use empty slice
		// This will return default values: false for inferred/verified, 0.0 for confidence
		var meta []*language.MetaEntry
		list = append(list, map[string]any{
			"name":       e.Name,
			"inferred":   brownfield.IsInferred(meta),
			"verified":   brownfield.IsVerified(meta),
			"confidence": brownfield.Confidence(meta),
		})
	}
	writeJSON(w, map[string]any{"entities": list})
}

func collectEntities(arch *language.Architecture) []*language.Entity {
	var out []*language.Entity
	if arch == nil {
		return out
	}
	out = append(out, arch.Entities...)
	for _, s := range arch.Systems {
		out = append(out, s.Entities...)
		for _, c := range s.Containers {
			out = append(out, c.Entities...)
		}
	}
	return out
}

func (s *Server) handleListEvents(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path string `json:"path"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	var list []map[string]string
	arch := prog.Architecture
	for _, ev := range collectEvents(arch) {
		v := ""
		if ev.Body != nil && ev.Body.Version != nil {
			v = *ev.Body.Version
		}
		list = append(list, map[string]string{"name": ev.Name, "version": v})
	}
	writeJSON(w, map[string]any{"events": list})
}

func collectEvents(arch *language.Architecture) []*language.DomainEvent {
	var out []*language.DomainEvent
	if arch == nil {
		return out
	}
	out = append(out, arch.Events...)
	for _, s := range arch.Systems {
		out = append(out, s.Events...)
		for _, c := range s.Containers {
			out = append(out, c.Events...)
		}
	}
	return out
}

func (s *Server) handleValidateEvent(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path  string `json:"path"`
		Event string `json:"event"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	prog, err := parseProgram(body.Path)
	if err != nil {
		writeErr(w, err)
		return
	}
	arch := prog.Architecture
	var target *language.DomainEvent
	for _, ev := range collectEvents(arch) {
		if ev.Name == body.Event {
			target = ev
			break
		}
	}
	if target == nil {
		writeErr(w, fmt.Errorf("event not found: %s", body.Event))
		return
	}
	// Basic lifecycle validation
	var result = map[string]any{"event": body.Event, "valid": true, "issues": []string{}}
	if target.Body != nil && target.Body.LifecycleEffect != nil {
		eff := target.Body.LifecycleEffect
		entName := eff.From.Entity
		fromState := eff.From.State
		toState := eff.To.State
		ent := findEntity(arch, entName)
		if ent == nil {
			result["valid"] = false
			result["issues"] = append(result["issues"].([]string), "entity not found: "+entName)
		} else {
			if !entityHasState(ent, fromState) || !entityHasState(ent, toState) || !entityHasTransition(ent, fromState, toState) {
				result["valid"] = false
				result["issues"] = append(result["issues"].([]string), "invalid lifecycle transition")
			}
		}
	}
	writeJSON(w, result)
}

func findEntity(arch *language.Architecture, name string) *language.Entity {
	if arch == nil {
		return nil
	}
	for _, e := range arch.Entities {
		if e.Name == name {
			return e
		}
	}
	for _, s := range arch.Systems {
		for _, e := range s.Entities {
			if e.Name == name {
				return e
			}
		}
		for _, c := range s.Containers {
			for _, e := range c.Entities {
				if e.Name == name {
					return e
				}
			}
		}
	}
	return nil
}

func entityHasState(e *language.Entity, state string) bool {
	if e == nil || e.Body == nil || e.Body.Lifecycle == nil {
		return false
	}
	for _, t := range e.Body.Lifecycle.Transitions {
		if t.From == state || t.To == state {
			return true
		}
	}
	return false
}

func entityHasTransition(e *language.Entity, from, to string) bool {
	if e == nil || e.Body == nil || e.Body.Lifecycle == nil {
		return false
	}
	for _, t := range e.Body.Lifecycle.Transitions {
		if t.From == from && t.To == to {
			return true
		}
	}
	return false
}

func (s *Server) handleValidateApprovalPolicy(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path      string            `json:"path"`
		Policy    any               `json:"policy"`
		PolicyDSL string            `json:"policy_dsl"`
		Changes   []approval.Change `json:"changes"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	// parse policy from JSON into our struct
	var pol approval.Policy
	if body.Policy != nil {
		data, _ := json.Marshal(body.Policy)
		json.Unmarshal(data, &pol)
	} else if strings.TrimSpace(body.PolicyDSL) != "" {
		parser, err := approval.BuildApprovalPolicyParser()
		if err == nil {
			pf, err := parser.ParseString("policy.approval", body.PolicyDSL)
			if err == nil && len(pf.Policies) > 0 {
				pol = approval.ToPolicy(pf.Policies[0])
			}
		}
	}
	// Evaluate
	res := approval.Evaluate(pol, body.Changes)
	writeJSON(w, res)
}

func (s *Server) handleValidateAllPolicies(w http.ResponseWriter, r *http.Request) {
	type req struct {
		Path       string            `json:"path"`
		Policies   any               `json:"policies"`
		PolicyDSLs []string          `json:"policy_dsls"`
		Changes    []approval.Change `json:"changes"`
	}
	var body req
	json.NewDecoder(r.Body).Decode(&body)
	// Build policy list
	var policies []approval.Policy
	if body.Policies != nil {
		data, _ := json.Marshal(body.Policies)
		json.Unmarshal(data, &policies)
	}
	if len(body.PolicyDSLs) > 0 {
		parser, err := approval.BuildApprovalPolicyParser()
		if err == nil {
			for _, dsl := range body.PolicyDSLs {
				pf, err := parser.ParseString("policy.approval", dsl)
				if err == nil {
					for _, pd := range pf.Policies {
						policies = append(policies, approval.ToPolicy(pd))
					}
				}
			}
		}
	}
	// Minimal IR placeholders; diff not computed here (changes provided)
	report := approval.EvaluateAll(nil, nil, nil, policies, body.Changes)
	writeJSON(w, report)
}
