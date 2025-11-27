// pkg/lsp/policy_rule_completion.go
package lsp

import (
	"strings"
)

// PolicyRuleCompletionProvider provides autocomplete for Policy and Rule DSLs.
type PolicyRuleCompletionProvider struct {
	index         *SemanticIndex
	registry      *MetadataRegistry
	mcpClient     MCPClient // Interface for MCP communication
	ruleRegistry  *RuleRegistry
	policyRegistry *PolicyRegistry
}

// MCPClient interface for communicating with MCP server to fetch dynamic data.
type MCPClient interface {
	ListRules() ([]RuleInfo, error)
	ListPolicies() ([]PolicyInfo, error)
	ListMetadataKeys() ([]string, error)
	ListFieldPaths(elementType string) ([]FieldPathInfo, error)
	ListADRs() ([]ADRInfo, error)
	ListStandards() ([]StandardInfo, error)
	ListPluginFields() ([]PluginFieldInfo, error)
}

// RuleInfo contains information about a rule for completion.
type RuleInfo struct {
	ID          string
	Description string
	AppliesTo   []string // element types
}

// PolicyInfo contains information about a policy for completion.
type PolicyInfo struct {
	ID          string
	Description string
}

// FieldPathInfo contains field path information for completion.
type FieldPathInfo struct {
	Path        string
	Type        string
	Description string
}

// ADRInfo contains ADR information for completion.
type ADRInfo struct {
	ID    string
	Title string
}

// StandardInfo contains standard information for completion.
type StandardInfo struct {
	ID          string
	Description string
}

// PluginFieldInfo contains plugin-extended field information.
type PluginFieldInfo struct {
	Path        string
	Type        string
	Description string
}

// RuleRegistry manages rule definitions.
type RuleRegistry struct {
	rules map[string]RuleInfo
}

// NewRuleRegistry creates a new rule registry.
func NewRuleRegistry() *RuleRegistry {
	return &RuleRegistry{
		rules: make(map[string]RuleInfo),
	}
}

// RegisterRule registers a rule in the registry.
func (rr *RuleRegistry) RegisterRule(rule RuleInfo) {
	rr.rules[rule.ID] = rule
}

// GetRule returns a rule by ID.
func (rr *RuleRegistry) GetRule(id string) (RuleInfo, bool) {
	rule, ok := rr.rules[id]
	return rule, ok
}

// GetAllRules returns all registered rules.
func (rr *RuleRegistry) GetAllRules() []RuleInfo {
	rules := make([]RuleInfo, 0, len(rr.rules))
	for _, rule := range rr.rules {
		rules = append(rules, rule)
	}
	return rules
}

// PolicyRegistry manages policy definitions.
type PolicyRegistry struct {
	policies map[string]PolicyInfo
}

// NewPolicyRegistry creates a new policy registry.
func NewPolicyRegistry() *PolicyRegistry {
	return &PolicyRegistry{
		policies: make(map[string]PolicyInfo),
	}
}

// RegisterPolicy registers a policy in the registry.
func (pr *PolicyRegistry) RegisterPolicy(policy PolicyInfo) {
	pr.policies[policy.ID] = policy
}

// GetPolicy returns a policy by ID.
func (pr *PolicyRegistry) GetPolicy(id string) (PolicyInfo, bool) {
	policy, ok := pr.policies[id]
	return policy, ok
}

// GetAllPolicies returns all registered policies.
func (pr *PolicyRegistry) GetAllPolicies() []PolicyInfo {
	policies := make([]PolicyInfo, 0, len(pr.policies))
	for _, policy := range pr.policies {
		policies = append(policies, policy)
	}
	return policies
}

// NewPolicyRuleCompletionProvider creates a new completion provider for Policy/Rule DSLs.
func NewPolicyRuleCompletionProvider(
	index *SemanticIndex,
	registry *MetadataRegistry,
	mcpClient MCPClient,
) *PolicyRuleCompletionProvider {
	return &PolicyRuleCompletionProvider{
		index:          index,
		registry:       registry,
		mcpClient:      mcpClient,
		ruleRegistry:   NewRuleRegistry(),
		policyRegistry: NewPolicyRegistry(),
	}
}

// ProvidePolicyCompletions provides completions for Policy DSL.
func (prcp *PolicyRuleCompletionProvider) ProvidePolicyCompletions(
	filePath string,
	text string,
	line, character int,
) ([]map[string]any, error) {
	lines := strings.Split(text, "\n")
	if line < 0 || line >= len(lines) {
		return nil, nil
	}

	currentLine := lines[line]
	beforeCursor := currentLine[:character]
	ctx := analyzePolicyContext(lines, line, character)

	var items []PolicyRuleCompletionItem

    switch ctx.ContextType {
    case ContextPolicyKeyword:
        items = prcp.providePolicyKeywordCompletions(ctx)
    case ContextPolicyBlock:
        items = prcp.providePolicyBlockCompletions(ctx)
    case ContextPolicyID:
        items = prcp.providePolicyIDCompletions(ctx)
    case ContextAppliesTo:
        items = prcp.provideAppliesToCompletions(ctx)
    case ContextWhenExpression:
        // reuse expression completions from rule provider
        items = prcp.provideExpressionCompletions(RuleContext{BeforeCursor: ctx.BeforeCursor})
    case ContextApprover:
        items = prcp.provideApproverCompletions(ctx)
    case ContextRuleID:
        items = prcp.provideRuleIDCompletions(ctx)
	case ContextRelatedADRs:
		items = prcp.provideADRCompletions(ctx)
	case ContextRelatedStandards:
		items = prcp.provideStandardCompletions(ctx)
	case ContextSeverity:
		items = prcp.provideSeverityCompletions(ctx)
	default:
		// Check if we're at the start of a policy declaration
		if strings.TrimSpace(beforeCursor) == "" || strings.HasSuffix(strings.TrimSpace(beforeCursor), "policy") {
			items = prcp.providePolicyKeywordCompletions(ctx)
		}
	}

	// Convert to LSP format
	result := make([]map[string]any, len(items))
	for i, item := range items {
		result[i] = item.ToLSPFormat()
	}

	return result, nil
}

// ProvideRuleCompletions provides completions for Rule DSL.
func (prcp *PolicyRuleCompletionProvider) ProvideRuleCompletions(
	filePath string,
	text string,
	line, character int,
) ([]map[string]any, error) {
	lines := strings.Split(text, "\n")
	if line < 0 || line >= len(lines) {
		return nil, nil
	}

	currentLine := lines[line]
	beforeCursor := currentLine[:character]
	ctx := analyzeRuleContext(lines, line, character)

	var items []PolicyRuleCompletionItem

	switch ctx.ContextType {
	case ContextRuleKeyword:
		items = prcp.provideRuleKeywordCompletions(ctx)
	case ContextRuleBlock:
		items = prcp.provideRuleBlockCompletions(ctx)
	case ContextRuleID:
		items = prcp.provideRuleIDCompletions(ctx)
	case ContextAppliesTo:
		items = prcp.provideAppliesToCompletions(ctx)
	case ContextWhenExpression:
		items = prcp.provideExpressionCompletions(ctx)
	case ContextEnsureExpression:
		items = prcp.provideExpressionCompletions(ctx)
	case ContextSuggestedFix:
		items = prcp.provideSuggestedFixCompletions(ctx)
	case ContextFieldPath:
		items = prcp.provideFieldPathCompletions(ctx)
	case ContextPolicyMetadataKey:
		items = prcp.provideMetadataKeyCompletions(ctx)
	case ContextSeverity:
		items = prcp.provideSeverityCompletions(ctx)
	default:
		// Check if we're at the start of a rule declaration
		if strings.TrimSpace(beforeCursor) == "" || strings.HasSuffix(strings.TrimSpace(beforeCursor), "rule") {
			items = prcp.provideRuleKeywordCompletions(ctx)
		}
	}

	// Convert to LSP format
	result := make([]map[string]any, len(items))
	for i, item := range items {
		result[i] = item.ToLSPFormat()
	}

	return result, nil
}

// PolicyContext represents the completion context for Policy DSL.
type PolicyContext struct {
	ContextType         PolicyRuleContextType
	InsidePolicyBlock   bool
	InsideRulesBlock    bool
	InsideControlsBlock bool
	CurrentField        string
	KeywordPrefix       string
	BeforeCursor        string
	Scope               string
}

// RuleContext represents the completion context for Rule DSL.
type RuleContext struct {
	ContextType       PolicyRuleContextType
	InsideRuleBlock   bool
	InsideWhenBlock   bool
	InsideEnsureBlock bool
	InsideFixBlock    bool
	CurrentField      string
	KeywordPrefix     string
	BeforeCursor      string
	Scope             string
	FieldPathPrefix   string
	MetadataKeyPrefix string
}

// PolicyRuleContextType indicates the type of Policy/Rule completion context.
type PolicyRuleContextType int

const (
	ContextPolicyKeyword PolicyRuleContextType = iota + 100
	ContextPolicyBlock
	ContextPolicyID
	ContextRuleKeyword
	ContextRuleBlock
	ContextRuleID
	ContextAppliesTo
	ContextWhenExpression
	ContextEnsureExpression
	ContextSuggestedFix
	ContextFieldPath
	ContextPolicyMetadataKey
	ContextRelatedADRs
	ContextRelatedStandards
    ContextSeverity
    ContextApprover
)

// analyzePolicyContext analyzes the context for Policy DSL completion.
func analyzePolicyContext(lines []string, line, character int) PolicyContext {
	ctx := PolicyContext{}
	if line < 0 || line >= len(lines) {
		return ctx
	}

	currentLine := lines[line]
	beforeCursor := currentLine[:character]

	// Check if we're inside a policy block
	for i := line; i >= 0; i-- {
		l := lines[i]
		if strings.Contains(l, "policy") && strings.Contains(l, "{") {
			ctx.InsidePolicyBlock = true
			break
		}
		if strings.Contains(l, "}") && !strings.Contains(l, "{") {
			break
		}
	}

	// Check if we're inside rules or controls block
	for i := line; i >= 0; i-- {
		l := lines[i]
		if strings.Contains(l, "rules {") {
			ctx.InsideRulesBlock = true
			break
		}
		if strings.Contains(l, "controls {") {
			ctx.InsideControlsBlock = true
			break
		}
		if strings.Contains(l, "}") && !strings.Contains(l, "{") {
			break
		}
	}

	// Determine context type
	trimmed := strings.TrimSpace(beforeCursor)
	if strings.HasSuffix(trimmed, "policy") || (trimmed == "" && line > 0 && strings.Contains(lines[line-1], "policy")) {
		ctx.ContextType = ContextPolicyKeyword
	} else if ctx.InsideRulesBlock && (strings.HasSuffix(trimmed, "") || strings.HasSuffix(trimmed, ",")) {
		ctx.ContextType = ContextRuleID
	} else if strings.Contains(beforeCursor, "applies_to") {
		ctx.ContextType = ContextAppliesTo
	} else if strings.Contains(beforeCursor, "related_adrs") {
		ctx.ContextType = ContextRelatedADRs
	} else if strings.Contains(beforeCursor, "related_standards") {
		ctx.ContextType = ContextRelatedStandards
	} else if strings.Contains(beforeCursor, "severity") {
		ctx.ContextType = ContextSeverity
	} else if ctx.InsidePolicyBlock {
		ctx.ContextType = ContextPolicyBlock
	}

	ctx.BeforeCursor = beforeCursor
	return ctx
}

// analyzeRuleContext analyzes the context for Rule DSL completion.
func analyzeRuleContext(lines []string, line, character int) RuleContext {
	ctx := RuleContext{}
	if line < 0 || line >= len(lines) {
		return ctx
	}

	currentLine := lines[line]
	beforeCursor := currentLine[:character]

	// Check if we're inside various blocks
	for i := line; i >= 0; i-- {
		l := lines[i]
		if strings.Contains(l, "rule") && strings.Contains(l, "{") {
			ctx.InsideRuleBlock = true
		}
		if strings.Contains(l, "when {") {
			ctx.InsideWhenBlock = true
		}
		if strings.Contains(l, "ensure {") {
			ctx.InsideEnsureBlock = true
		}
		if strings.Contains(l, "suggested_fix {") {
			ctx.InsideFixBlock = true
		}
		if strings.Contains(l, "}") && !strings.Contains(l, "{") {
			break
		}
	}

	// Determine context type
	trimmed := strings.TrimSpace(beforeCursor)
	if strings.HasSuffix(trimmed, "rule") {
		ctx.ContextType = ContextRuleKeyword
	} else if strings.Contains(beforeCursor, "applies_to") {
		ctx.ContextType = ContextAppliesTo
		} else if ctx.InsideWhenBlock {
		ctx.ContextType = ContextWhenExpression
		if strings.Contains(beforeCursor, "metadata.") {
			ctx.ContextType = ContextPolicyMetadataKey
			parts := strings.Split(beforeCursor, "metadata.")
			if len(parts) > 1 {
				ctx.MetadataKeyPrefix = strings.TrimSpace(parts[1])
			}
		} else if strings.Contains(beforeCursor, ".") {
			ctx.ContextType = ContextFieldPath
			parts := strings.Split(beforeCursor, ".")
			if len(parts) > 1 {
				ctx.FieldPathPrefix = strings.TrimSpace(parts[len(parts)-1])
			}
		}
	} else if ctx.InsideEnsureBlock {
		ctx.ContextType = ContextEnsureExpression
	} else if ctx.InsideFixBlock {
		ctx.ContextType = ContextSuggestedFix
	} else if strings.Contains(beforeCursor, "severity") {
		ctx.ContextType = ContextSeverity
	} else if ctx.InsideRuleBlock {
		ctx.ContextType = ContextRuleBlock
	}

	ctx.BeforeCursor = beforeCursor
	return ctx
}

// providePolicyKeywordCompletions provides keyword completions for Policy DSL.
func (prcp *PolicyRuleCompletionProvider) providePolicyKeywordCompletions(ctx PolicyContext) []PolicyRuleCompletionItem {
	return []PolicyRuleCompletionItem{
		{
			Label:      "policy",
			Kind:       14, // Keyword
			InsertText: "policy ${1:policyId} {\n  description \"${2:Policy description}\"\n  applies_to ${3:elementType}\n  \n  rules {\n    ${4:ruleId}\n  }\n  \n  controls {\n    ${5}\n  }\n  \n  severity ${6:warning}\n}",
			Detail:     "Define a new policy",
			Documentation: "Policy DSL structure for governance and compliance",
		},
	}
}

// providePolicyBlockCompletions provides completions inside a policy block.
func (prcp *PolicyRuleCompletionProvider) providePolicyBlockCompletions(ctx PolicyContext) []PolicyRuleCompletionItem {
    items := []PolicyRuleCompletionItem{
		{
			Label:      "description",
			Kind:       5, // Property
			InsertText: "description \"${1:Description}\"",
			Detail:     "Policy description",
		},
        {
            Label:      "applies_to",
            Kind:       5,
            InsertText: "applies_to: ${1:system} ${2:Target}",
            Detail:     "Targets this policy applies to",
        },
        {
            Label:      "when",
            Kind:       5,
            InsertText: "when: ${1:left} ${2:operator} ${3:right}",
            Detail:     "Condition under which approval is required",
        },
        {
            Label:      "require",
            Kind:       5,
            InsertText: "require: ${1:\"security_team\"}",
            Detail:     "Approver or approver group",
        },
        {
            Label:      "auto_approve",
            Kind:       5,
            InsertText: "auto_approve: ${1:true}",
            Detail:     "Auto-approve safe changes",
        },
        {
            Label:      "except",
            Kind:       5,
            InsertText: "except: when ${1:branch.name startswith \"experiment/\"}",
            Detail:     "Exceptions that skip approval",
        },
        {
            Label:      "related_standards",
            Kind:       5,
            InsertText: "related_standards [\n    \"${1:standardId}\"\n  ]",
            Detail:     "Related standards",
        },
		{
			Label:      "related_adrs",
			Kind:       5,
			InsertText: "related_adrs [\n    \"${1:adrId}\"\n  ]",
			Detail:     "Related ADRs",
		},
        {
            Label:      "severity",
            Kind:       5,
            InsertText: "severity: ${1:error}",
            Detail:     "Severity level",
        },
		{
			Label:      "owner",
			Kind:       5,
			InsertText: "owner \"${1:owner}\"",
			Detail:     "Policy owner",
		},
		{
			Label:      "version",
			Kind:       5,
			InsertText: "version \"${1:1.0.0}\"",
			Detail:     "Policy version",
		},
		{
			Label:      "remediation",
			Kind:       5,
			InsertText: "remediation \"${1:Remediation steps}\"",
			Detail:     "Remediation guidance",
		},
	}
	return items
}

// providePolicyIDCompletions provides policy ID completions.
func (prcp *PolicyRuleCompletionProvider) providePolicyIDCompletions(ctx PolicyContext) []PolicyRuleCompletionItem {
	var items []PolicyRuleCompletionItem

	// Get policies from registry
	policies := prcp.policyRegistry.GetAllPolicies()

	// Also try to get from MCP if available
	if prcp.mcpClient != nil {
		mcpPolicies, err := prcp.mcpClient.ListPolicies()
		if err == nil {
			for _, p := range mcpPolicies {
				policies = append(policies, PolicyInfo{ID: p.ID, Description: p.Description})
			}
		}
	}

	prefix := lastWordPolicyRule(ctx.BeforeCursor)
	for _, policy := range policies {
		if prefix == "" || strings.HasPrefix(policy.ID, prefix) {
			items = append(items, PolicyRuleCompletionItem{
				Label:         policy.ID,
				Kind:          7, // Value
				InsertText:    policy.ID,
				Detail:        policy.Description,
				Documentation: policy.Description,
			})
		}
	}

	return items
}

// provideRuleKeywordCompletions provides keyword completions for Rule DSL.
func (prcp *PolicyRuleCompletionProvider) provideRuleKeywordCompletions(ctx RuleContext) []PolicyRuleCompletionItem {
	return []PolicyRuleCompletionItem{
		{
			Label:      "rule",
			Kind:       14, // Keyword
			InsertText: "rule ${1:ruleId} {\n  description \"${2:Rule description}\"\n  \n  applies_to ${3:elementType}\n  \n  when {\n    ${4:condition}\n  }\n  \n  ensure {\n    ${5:assertion}\n  }\n  \n  severity ${6:error}\n  \n  message \"${7:Violation message}\"\n}",
			Detail:     "Define a new rule",
			Documentation: "Rule DSL structure for validation",
		},
	}
}

// provideRuleBlockCompletions provides completions inside a rule block.
func (prcp *PolicyRuleCompletionProvider) provideRuleBlockCompletions(ctx RuleContext) []PolicyRuleCompletionItem {
	items := []PolicyRuleCompletionItem{
		{
			Label:      "description",
			Kind:       5, // Property
			InsertText: "description \"${1:Description}\"",
			Detail:     "Rule description",
		},
		{
			Label:      "applies_to",
			Kind:       5,
			InsertText: "applies_to ${1:elementType}",
			Detail:     "Element types this rule applies to",
		},
		{
			Label:      "when",
			Kind:       5,
			InsertText: "when {\n    ${1:condition}\n  }",
			Detail:     "Condition for rule to apply",
		},
		{
			Label:      "ensure",
			Kind:       5,
			InsertText: "ensure {\n    ${1:assertion}\n  }",
			Detail:     "Assertion that must be true",
		},
		{
			Label:      "severity",
			Kind:       5,
			InsertText: "severity ${1:error}",
			Detail:     "Severity level",
		},
		{
			Label:      "message",
			Kind:       5,
			InsertText: "message \"${1:Violation message}\"",
			Detail:     "Error message for violations",
		},
		{
			Label:      "suggested_fix",
			Kind:       5,
			InsertText: "suggested_fix {\n    ${1}\n  }",
			Detail:     "Suggested fix for violations",
		},
	}
	return items
}

// provideAppliesToCompletions provides element type completions for applies_to.
func (prcp *PolicyRuleCompletionProvider) provideAppliesToCompletions(ctx interface{}) []PolicyRuleCompletionItem {
    elementTypes := []string{
        "architecture",
        "domain",
        "system",
        "container",
        "component",
        "entity",
        "field",
        "event",
        "contract",
        "diagram",
    }

	var items []PolicyRuleCompletionItem
	for _, et := range elementTypes {
		items = append(items, PolicyRuleCompletionItem{
			Label:      et,
			Kind:       7, // Value
			InsertText: et,
			Detail:     "Element type: " + et,
		})
	}
	return items
}

// provideApproverCompletions suggests approver actors and groups for Approval Policy DSL
func (prcp *PolicyRuleCompletionProvider) provideApproverCompletions(ctx PolicyContext) []PolicyRuleCompletionItem {
    approvers := []string{
        "security_team",
        "platform_architect",
        "domain_architect_payments",
        "compliance_team",
        "cto",
        "data_steward",
    }
    var items []PolicyRuleCompletionItem
    for _, a := range approvers {
        items = append(items, PolicyRuleCompletionItem{
            Label:      a,
            Kind:       7,
            InsertText: "\"" + a + "\"",
            Detail:     "Approver",
        })
    }
    // group quorum snippet
    items = append(items, PolicyRuleCompletionItem{
        Label:      "group",
        Kind:       14,
        InsertText: "{\n  group: \"${1:group}\",\n  count: ${2:1}\n}",
        Detail:     "Approver group quorum",
    })
    return items
}

// provideRuleIDCompletions provides rule ID completions.
func (prcp *PolicyRuleCompletionProvider) provideRuleIDCompletions(ctx interface{}) []PolicyRuleCompletionItem {
	var items []PolicyRuleCompletionItem

	// Get rules from registry
	rules := prcp.ruleRegistry.GetAllRules()

	// Also try to get from MCP if available
	if prcp.mcpClient != nil {
		mcpRules, err := prcp.mcpClient.ListRules()
		if err == nil {
			for _, r := range mcpRules {
				rules = append(rules, RuleInfo{ID: r.ID, Description: r.Description})
			}
		}
	}

	// Common built-in rules
	builtInRules := []RuleInfo{
		{ID: "noDirectDBAccess", Description: "Prevent direct database access from web components"},
		{ID: "require_rate_limit", Description: "Require rate limiting for public APIs"},
		{ID: "noInternalImports", Description: "Prevent importing internal systems externally"},
		{ID: "criticalityRequiresSLO", Description: "Critical systems must have SLO defined"},
	}

	rules = append(rules, builtInRules...)

	for _, rule := range rules {
		items = append(items, PolicyRuleCompletionItem{
			Label:         rule.ID,
			Kind:          7, // Value
			InsertText:    rule.ID,
			Detail:        rule.Description,
			Documentation: rule.Description,
		})
	}

	return items
}

// provideExpressionCompletions provides completions for expression DSL.
func (prcp *PolicyRuleCompletionProvider) provideExpressionCompletions(ctx RuleContext) []PolicyRuleCompletionItem {
	items := []PolicyRuleCompletionItem{}

	// Operators
	operators := []struct {
		label      string
		insertText string
		detail     string
	}{
		{"==", " == ", "Equality operator"},
		{"!=", " != ", "Inequality operator"},
		{"in", " in ", "Contains operator"},
		{"not in", " not in ", "Not contains operator"},
		{"contains", " contains ", "String contains"},
		{"starts_with", " starts_with ", "String starts with"},
		{"ends_with", " ends_with ", "String ends with"},
		{"matches", " matches ", "Regex match"},
		{"exists", " exists", "Check existence"},
		{"not exists", " not exists", "Check non-existence"},
	}

	beforeCursor := ctx.BeforeCursor
	trimmed := strings.TrimSpace(beforeCursor)

	// If we're after a field path or value, suggest operators
	if strings.Contains(trimmed, ".") || strings.HasPrefix(trimmed, "\"") {
		for _, op := range operators {
			items = append(items, PolicyRuleCompletionItem{
				Label:      op.label,
				Kind:       13, // Operator
				InsertText: op.insertText,
				Detail:     op.detail,
			})
		}
	}

	// Field path completions
	if strings.Contains(beforeCursor, ".") {
		items = append(items, prcp.provideFieldPathCompletions(ctx)...)
	}

	// Metadata key completions (using ContextPolicyMetadataKey)
	if strings.Contains(beforeCursor, "metadata.") {
		ctx.ContextType = ContextPolicyMetadataKey
		items = append(items, prcp.provideMetadataKeyCompletions(ctx)...)
	}

	return items
}

// provideFieldPathCompletions provides field path completions.
func (prcp *PolicyRuleCompletionProvider) provideFieldPathCompletions(ctx RuleContext) []PolicyRuleCompletionItem {
	var items []PolicyRuleCompletionItem

	scopes := []string{
		"metadata",
		"from",
		"to",
		"system",
		"container",
		"component",
		"relation",
	}

	// Get field paths from MCP if available
	var fieldPaths []FieldPathInfo
	if prcp.mcpClient != nil {
		for _, scope := range scopes {
			paths, err := prcp.mcpClient.ListFieldPaths(scope)
			if err == nil {
				fieldPaths = append(fieldPaths, paths...)
			}
		}
	}

	// Common field paths
	commonFields := map[string][]string{
		"system":    {"id", "label", "description", "technology", "metadata.*"},
		"container": {"id", "label", "description", "technology", "metadata.*", "tags"},
		"component": {"id", "label", "description", "technology", "metadata.*"},
		"relation":  {"from", "to", "verb", "label", "metadata.*"},
		"metadata":  {}, // Handled separately
	}

	prefix := ctx.FieldPathPrefix
	for scope, fields := range commonFields {
		for _, field := range fields {
			fullPath := scope + "." + field
			if prefix == "" || strings.HasPrefix(field, prefix) {
				items = append(items, PolicyRuleCompletionItem{
					Label:      fullPath,
					Kind:       5, // Property
					InsertText: fullPath,
					Detail:     "Field path: " + fullPath,
				})
			}
		}
	}

	// Add MCP-provided field paths
	for _, fp := range fieldPaths {
		if prefix == "" || strings.HasPrefix(fp.Path, prefix) {
			items = append(items, PolicyRuleCompletionItem{
				Label:      fp.Path,
				Kind:       5,
				InsertText: fp.Path,
				Detail:     fp.Description,
			})
		}
	}

	return items
}

// provideMetadataKeyCompletions provides metadata key completions.
func (prcp *PolicyRuleCompletionProvider) provideMetadataKeyCompletions(ctx RuleContext) []PolicyRuleCompletionItem {
	var items []PolicyRuleCompletionItem

	// Get from metadata registry
	if prcp.registry != nil {
		descriptors := prcp.registry.ForScope(ctx.Scope)
		prefix := ctx.MetadataKeyPrefix
		for _, desc := range descriptors {
			if prefix == "" || strings.HasPrefix(desc.Key, prefix) {
				items = append(items, PolicyRuleCompletionItem{
					Label:      desc.Key,
					Kind:       5, // Property
					InsertText: desc.Key,
					Detail:     desc.Description,
				})
			}
		}
	}

	// Get from MCP if available
	if prcp.mcpClient != nil {
		keys, err := prcp.mcpClient.ListMetadataKeys()
		if err == nil {
			prefix := ctx.MetadataKeyPrefix
			for _, key := range keys {
				if prefix == "" || strings.HasPrefix(key, prefix) {
					items = append(items, PolicyRuleCompletionItem{
						Label:      key,
						Kind:       5,
						InsertText: key,
						Detail:     "Metadata key: " + key,
					})
				}
			}
		}
	}

	return items
}

// provideSeverityCompletions provides severity level completions.
func (prcp *PolicyRuleCompletionProvider) provideSeverityCompletions(ctx interface{}) []PolicyRuleCompletionItem {
	severities := []string{"info", "warning", "error", "critical"}

	var items []PolicyRuleCompletionItem
	for _, sev := range severities {
		items = append(items, PolicyRuleCompletionItem{
			Label:      sev,
			Kind:       7, // Value
			InsertText: sev,
			Detail:     "Severity: " + sev,
		})
	}
	return items
}

// provideADRCompletions provides ADR ID completions.
func (prcp *PolicyRuleCompletionProvider) provideADRCompletions(ctx PolicyContext) []PolicyRuleCompletionItem {
	var items []PolicyRuleCompletionItem

	// Get ADRs from semantic index
	if prcp.index != nil {
		adrs := prcp.index.GetADRs()
		for _, adr := range adrs {
			items = append(items, PolicyRuleCompletionItem{
				Label:      adr,
				Kind:       7, // Value
				InsertText: "\"" + adr + "\"",
				Detail:     "ADR: " + adr,
			})
		}
	}

	// Get from MCP if available
	if prcp.mcpClient != nil {
		mcpADRs, err := prcp.mcpClient.ListADRs()
		if err == nil {
			for _, adr := range mcpADRs {
				items = append(items, PolicyRuleCompletionItem{
					Label:      adr.ID,
					Kind:       7,
					InsertText: "\"" + adr.ID + "\"",
					Detail:     adr.Title,
				})
			}
		}
	}

	return items
}

// provideStandardCompletions provides standard ID completions.
func (prcp *PolicyRuleCompletionProvider) provideStandardCompletions(ctx PolicyContext) []PolicyRuleCompletionItem {
	var items []PolicyRuleCompletionItem

	// Get from MCP if available
	if prcp.mcpClient != nil {
		standards, err := prcp.mcpClient.ListStandards()
		if err == nil {
			for _, std := range standards {
				items = append(items, PolicyRuleCompletionItem{
					Label:      std.ID,
					Kind:       7, // Value
					InsertText: "\"" + std.ID + "\"",
					Detail:     std.Description,
				})
			}
		}
	}

	return items
}

// provideSuggestedFixCompletions provides suggested fix command completions.
func (prcp *PolicyRuleCompletionProvider) provideSuggestedFixCompletions(ctx RuleContext) []PolicyRuleCompletionItem {
	fixCommands := []struct {
		label      string
		insertText string
		detail     string
	}{
		{
			label:      "add_metadata",
			insertText: "add_metadata ${1:key} = \"${2:value}\"",
			detail:     "Add metadata to element",
		},
		{
			label:      "remove_metadata",
			insertText: "remove_metadata ${1:key}",
			detail:     "Remove metadata from element",
		},
		{
			label:      "replace_import",
			insertText: "replace_import \"${1:old}\" with \"${2:new}\"",
			detail:     "Replace import statement",
		},
		{
			label:      "add_relation",
			insertText: "add_relation ${1:from} -> ${2:to}",
			detail:     "Add relation",
		},
		{
			label:      "remove_relation",
			insertText: "remove_relation ${1:from} -> ${2:to}",
			detail:     "Remove relation",
		},
		{
			label:      "set",
			insertText: "set ${1:field} = \"${2:value}\"",
			detail:     "Set field value",
		},
	}

	var items []PolicyRuleCompletionItem
	for _, cmd := range fixCommands {
		items = append(items, PolicyRuleCompletionItem{
			Label:      cmd.label,
			Kind:       14, // Keyword
			InsertText: cmd.insertText,
			Detail:     cmd.detail,
		})
	}

	return items
}

// Helper function to get last word from string (for policy/rule context).
func lastWordPolicyRule(s string) string {
	parts := strings.Fields(s)
	if len(parts) == 0 {
		return ""
	}
	return parts[len(parts)-1]
}

// PolicyRuleCompletionItem represents a single completion suggestion for Policy/Rule DSL.
type PolicyRuleCompletionItem struct {
	Label         string
	Kind          int // LSP CompletionItemKind
	InsertText    string
	Detail        string
	Documentation string
}

// ToLSPFormat converts PolicyRuleCompletionItem to LSP format.
func (ci PolicyRuleCompletionItem) ToLSPFormat() map[string]any {
	result := map[string]any{
		"label": ci.Label,
		"kind":  ci.Kind,
	}
	if ci.InsertText != "" {
		result["insertText"] = ci.InsertText
	}
	if ci.Detail != "" {
		result["detail"] = ci.Detail
	}
	if ci.Documentation != "" {
		result["documentation"] = ci.Documentation
	}
	return result
}
