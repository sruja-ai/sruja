// pkg/export/markdown/faang_helpers.go
// Package markdown provides helper functions for FAANG-level sections:
// Capacity Planning, Monitoring, Security Threat Model, Compliance, Dependencies,
// Cost Analysis, API Versioning, Multi-Region, Data Lifecycle.
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func (e *Exporter) writeCapacityPlanning(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Capacity Planning\n\n")

	// Current Capacity
	sb.WriteString("### Current Capacity\n\n")
	hasCapacity := false

	// Extract from deployment nodes
	if len(arch.DeploymentNodes) > 0 {
		for _, deployment := range arch.DeploymentNodes {
			if deployment.Label != "" {
				sb.WriteString(fmt.Sprintf("- **%s**: ", deployment.Label))
			} else {
				sb.WriteString(fmt.Sprintf("- **%s**: ", deployment.ID))
			}

			// Extract instance count from container instances
			if len(deployment.ContainerInstances) > 0 {
				sb.WriteString(fmt.Sprintf("%d container instances", len(deployment.ContainerInstances)))
				hasCapacity = true
			} else {
				sb.WriteString("Not specified")
			}
			sb.WriteString("\n")
		}
	}

	// Extract from system/container metadata
	for _, sys := range arch.Systems {
		if capacity, ok := sys.MetaString("capacity"); ok {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", sys.Label, capacity))
			hasCapacity = true
		}
		for _, cont := range sys.Containers {
			if capacity, ok := cont.MetaString("capacity"); ok {
				sb.WriteString(fmt.Sprintf("- **%s**: %s\n", cont.Label, capacity))
				hasCapacity = true
			}
		}
	}

	if !hasCapacity {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Scaling Strategy
	sb.WriteString("### Scaling Strategy\n\n")
	hasScaling := false

	// Check for auto-scaling from metadata
	for _, sys := range arch.Systems {
		if scaling, ok := sys.MetaString("scaling"); ok {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", sys.Label, scaling))
			hasScaling = true
		}
		if scaling, ok := sys.MetaString("autoscaling"); ok {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", sys.Label, scaling))
			hasScaling = true
		}
	}

	// Check for read replicas (datastores)
	for _, sys := range arch.Systems {
		for _, ds := range sys.DataStores {
			if replicas, ok := ds.MetaString("readReplicas"); ok {
				sb.WriteString(fmt.Sprintf("- **%s**: %s read replicas\n", ds.Label, replicas))
				hasScaling = true
			}
		}
	}

	if !hasScaling {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Projected Growth
	sb.WriteString("### Projected Growth\n\n")
	hasProjection := false

	if projection, ok := arch.MetaString("projectedGrowth"); ok {
		sb.WriteString(projection + "\n")
		hasProjection = true
	}

	if !hasProjection {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Bottlenecks
	sb.WriteString("### Bottlenecks\n\n")
	hasBottlenecks := false

	if bottlenecks, ok := arch.MetaString("bottlenecks"); ok {
		sb.WriteString(bottlenecks + "\n")
		hasBottlenecks = true
	}

	if !hasBottlenecks {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeMonitoringObservability(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Monitoring & Observability\n\n")

	// Metrics
	sb.WriteString("### Metrics\n\n")
	hasMetrics := false

	if metrics, ok := arch.MetaString("metrics"); ok {
		sb.WriteString(fmt.Sprintf("- **Application Metrics**: %s\n", metrics))
		hasMetrics = true
	}
	if infraMetrics, ok := arch.MetaString("infrastructureMetrics"); ok {
		sb.WriteString(fmt.Sprintf("- **Infrastructure Metrics**: %s\n", infraMetrics))
		hasMetrics = true
	}

	if !hasMetrics {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Key Dashboards
	sb.WriteString("### Key Dashboards\n\n")
	hasDashboards := false

	if dashboards, ok := arch.MetaString("dashboards"); ok {
		sb.WriteString(dashboards + "\n")
		hasDashboards = true
	}

	if !hasDashboards {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Alerting
	sb.WriteString("### Alerting\n\n")
	hasAlerting := false

	if alerting, ok := arch.MetaString("alerting"); ok {
		sb.WriteString(alerting + "\n")
		hasAlerting = true
	}

	if !hasAlerting {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Logging
	sb.WriteString("### Logging\n\n")
	hasLogging := false

	if logging, ok := arch.MetaString("logging"); ok {
		sb.WriteString(logging + "\n")
		hasLogging = true
	}

	if !hasLogging {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Tracing
	sb.WriteString("### Tracing\n\n")
	hasTracing := false

	if tracing, ok := arch.MetaString("tracing"); ok {
		sb.WriteString(tracing + "\n")
		hasTracing = true
	}

	if !hasTracing {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeSecurityThreatModel(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Security Threat Model\n\n")

	// Threat Analysis
	sb.WriteString("### Threat Analysis\n\n")

	// STRIDE Analysis
	sb.WriteString("#### STRIDE Analysis\n\n")
	strideItems := []string{}

	// Check security requirements and policies
	for _, req := range arch.Requirements {
		if req.Type != nil && *req.Type == "security" {
			if req.Description != nil {
				strideItems = append(strideItems, *req.Description)
			}
		}
	}

	for _, policy := range arch.Policies {
		if policy.Category != nil && strings.Contains(strings.ToLower(*policy.Category), "security") {
			strideItems = append(strideItems, policy.Description)
		}
	}

	if len(strideItems) > 0 {
		for _, item := range strideItems {
			sb.WriteString(fmt.Sprintf("- %s\n", item))
		}
	} else {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Attack Vectors
	sb.WriteString("### Attack Vectors\n\n")
	hasVectors := false

	if vectors, ok := arch.MetaString("attackVectors"); ok {
		sb.WriteString(vectors + "\n")
		hasVectors = true
	}

	if !hasVectors {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Security Controls
	sb.WriteString("### Security Controls\n\n")

	controls := []string{}

	// Extract from security requirements
	for _, req := range arch.Requirements {
		if req.Type != nil && *req.Type == "security" {
			if req.Description != nil {
				controls = append(controls, *req.Description)
			}
		}
	}

	// Extract from security policies
	for _, policy := range arch.Policies {
		if policy.Category != nil && strings.Contains(strings.ToLower(*policy.Category), "security") {
			controls = append(controls, policy.Description)
		}
	}

	if len(controls) > 0 {
		for _, control := range controls {
			sb.WriteString(fmt.Sprintf("- %s\n", control))
		}
	} else {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeComplianceMatrix(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Compliance & Certifications\n\n")

	// Compliance Status
	sb.WriteString("### Compliance Status\n\n")
	hasCompliance := false

	// Check for compliance in requirements
	complianceReqs := []string{}
	for _, req := range arch.Requirements {
		if req.Type != nil && *req.Type == "constraint" {
			if req.Description != nil && (strings.Contains(strings.ToLower(*req.Description), "pci") ||
				strings.Contains(strings.ToLower(*req.Description), "gdpr") ||
				strings.Contains(strings.ToLower(*req.Description), "soc") ||
				strings.Contains(strings.ToLower(*req.Description), "hipaa") ||
				strings.Contains(strings.ToLower(*req.Description), "iso")) {
				complianceReqs = append(complianceReqs, *req.Description)
				hasCompliance = true
			}
		}
	}

	// Check metadata
	if pci, ok := arch.MetaString("pciDss"); ok {
		sb.WriteString(fmt.Sprintf("- **PCI-DSS**: %s\n", pci))
		hasCompliance = true
	}
	if soc2, ok := arch.MetaString("soc2"); ok {
		sb.WriteString(fmt.Sprintf("- **SOC 2**: %s\n", soc2))
		hasCompliance = true
	}
	if gdpr, ok := arch.MetaString("gdpr"); ok {
		sb.WriteString(fmt.Sprintf("- **GDPR**: %s\n", gdpr))
		hasCompliance = true
	}
	if hipaa, ok := arch.MetaString("hipaa"); ok {
		sb.WriteString(fmt.Sprintf("- **HIPAA**: %s\n", hipaa))
		hasCompliance = true
	}
	if iso, ok := arch.MetaString("iso27001"); ok {
		sb.WriteString(fmt.Sprintf("- **ISO 27001**: %s\n", iso))
		hasCompliance = true
	}

	if len(complianceReqs) > 0 {
		for _, req := range complianceReqs {
			sb.WriteString(fmt.Sprintf("- %s\n", req))
		}
	}

	if !hasCompliance {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Compliance Controls
	sb.WriteString("### Compliance Controls\n\n")
	hasControls := false

	if controls, ok := arch.MetaString("complianceControls"); ok {
		sb.WriteString(controls + "\n")
		hasControls = true
	}

	if !hasControls {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeDependencyRiskAssessment(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Dependency Risk Assessment\n\n")

	// External Dependencies
	sb.WriteString("### External Dependencies\n\n")
	hasExternal := false

	for _, sys := range arch.Systems {
		isExternal := false
		for _, meta := range sys.Metadata {
			if meta.Key == "tags" && meta.Value != nil && strings.Contains(*meta.Value, "external") {
				isExternal = true
				break
			}
		}

		if isExternal {
			sysName := sys.Label
			if sysName == "" {
				sysName = sys.ID
			}

			sb.WriteString(fmt.Sprintf("#### %s\n", sysName))

			// Risk Level
			riskLevel := "🟡 Medium"
			if risk, ok := sys.MetaString("riskLevel"); ok {
				if strings.Contains(strings.ToLower(risk), "high") {
					riskLevel = "🔴 High"
				} else if strings.Contains(strings.ToLower(risk), "low") {
					riskLevel = "🟢 Low"
				}
			}
			sb.WriteString(fmt.Sprintf("- **Risk Level**: %s\n", riskLevel))

			// Mitigation
			mitigation := "Not specified"
			if mit, ok := sys.MetaString("mitigation"); ok {
				mitigation = mit
			}
			sb.WriteString(fmt.Sprintf("- **Mitigation**: %s\n", mitigation))

			// SLA
			sla := "Not specified"
			if slaMeta, ok := sys.MetaString("sla"); ok {
				sla = slaMeta
			}
			sb.WriteString(fmt.Sprintf("- **SLA**: %s\n", sla))

			// Impact
			impact := "Service dependency unavailable"
			if impactMeta, ok := sys.MetaString("impact"); ok {
				impact = impactMeta
			}
			sb.WriteString(fmt.Sprintf("- **Impact**: %s\n", impact))

			sb.WriteString("\n")
			hasExternal = true
		}
	}

	if !hasExternal {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Internal Dependencies
	sb.WriteString("### Internal Dependencies\n\n")
	hasInternal := false

	// Check for critical internal dependencies
	for _, sys := range arch.Systems {
		// Skip external systems
		isExternal := false
		for _, meta := range sys.Metadata {
			if meta.Key == "tags" && meta.Value != nil && strings.Contains(*meta.Value, "external") {
				isExternal = true
				break
			}
		}
		if isExternal {
			continue
		}

		// Check if system has critical dependencies
		if len(sys.DataStores) > 0 || len(sys.Queues) > 0 {
			sysName := sys.Label
			if sysName == "" {
				sysName = sys.ID
			}

			deps := []string{}
			if len(sys.DataStores) > 0 {
				deps = append(deps, fmt.Sprintf("%d data stores", len(sys.DataStores)))
			}
			if len(sys.Queues) > 0 {
				deps = append(deps, fmt.Sprintf("%d message queues", len(sys.Queues)))
			}

			if len(deps) > 0 {
				sb.WriteString(fmt.Sprintf("- **%s**: %s\n", sysName, strings.Join(deps, ", ")))
				hasInternal = true
			}
		}
	}

	if !hasInternal {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeCostAnalysis(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Cost Analysis\n\n")

	// Monthly Operating Costs
	sb.WriteString("### Monthly Operating Costs\n\n")
	hasCosts := false

	if cost, ok := arch.MetaString("monthlyCost"); ok {
		sb.WriteString(fmt.Sprintf("**Total**: %s\n\n", cost))
		hasCosts = true
	}

	// Extract cost breakdown from metadata
	costBreakdown := []string{}
	if compute, ok := arch.MetaString("cost.compute"); ok {
		costBreakdown = append(costBreakdown, fmt.Sprintf("- **Compute**: %s", compute))
		hasCosts = true
	}
	if database, ok := arch.MetaString("cost.database"); ok {
		costBreakdown = append(costBreakdown, fmt.Sprintf("- **Database**: %s", database))
		hasCosts = true
	}
	if storage, ok := arch.MetaString("cost.storage"); ok {
		costBreakdown = append(costBreakdown, fmt.Sprintf("- **Storage**: %s", storage))
		hasCosts = true
	}
	if network, ok := arch.MetaString("cost.network"); ok {
		costBreakdown = append(costBreakdown, fmt.Sprintf("- **Network**: %s", network))
		hasCosts = true
	}
	if monitoring, ok := arch.MetaString("cost.monitoring"); ok {
		costBreakdown = append(costBreakdown, fmt.Sprintf("- **Monitoring**: %s", monitoring))
		hasCosts = true
	}

	if len(costBreakdown) > 0 {
		for _, item := range costBreakdown {
			sb.WriteString(item + "\n")
		}
		sb.WriteString("\n")
	}

	if !hasCosts {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Cost per Transaction
	sb.WriteString("### Cost per Transaction\n\n")
	hasCostPerTx := false

	if costPerTx, ok := arch.MetaString("costPerTransaction"); ok {
		sb.WriteString(fmt.Sprintf("- **Average**: %s\n", costPerTx))
		hasCostPerTx = true
	}
	if costBreakdown, ok := arch.MetaString("costBreakdown"); ok {
		sb.WriteString(fmt.Sprintf("- **Breakdown**: %s\n", costBreakdown))
		hasCostPerTx = true
	}

	if !hasCostPerTx {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Cost Optimization
	sb.WriteString("### Cost Optimization\n\n")
	hasOptimization := false

	if optimization, ok := arch.MetaString("costOptimization"); ok {
		sb.WriteString(optimization + "\n")
		hasOptimization = true
	}
	if savings, ok := arch.MetaString("costSavings"); ok {
		sb.WriteString(fmt.Sprintf("- **Projected Savings**: %s\n", savings))
		hasOptimization = true
	}

	if !hasOptimization {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeAPIVersioning(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## API Versioning\n\n")

	// Versioning Strategy
	sb.WriteString("### Versioning Strategy\n\n")
	hasStrategy := false

	if strategy, ok := arch.MetaString("apiVersioningStrategy"); ok {
		sb.WriteString(fmt.Sprintf("- **Approach**: %s\n", strategy))
		hasStrategy = true
	}
	if deprecation, ok := arch.MetaString("apiDeprecationPolicy"); ok {
		sb.WriteString(fmt.Sprintf("- **Deprecation Policy**: %s\n", deprecation))
		hasStrategy = true
	}

	// Check contracts for version information
	for _, contract := range arch.Contracts {
		if contract.Kind == "api" && contract.Body != nil {
			if contract.Body.Version != nil {
				sb.WriteString(fmt.Sprintf("- **Current Version**: %s\n", *contract.Body.Version))
				hasStrategy = true
			}
		}
	}

	if !hasStrategy {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Version Lifecycle
	sb.WriteString("### Version Lifecycle\n\n")
	hasLifecycle := false

	if lifecycle, ok := arch.MetaString("apiVersionLifecycle"); ok {
		sb.WriteString(lifecycle + "\n")
		hasLifecycle = true
	}

	// Extract from contracts
	versions := []string{}
	for _, contract := range arch.Contracts {
		if contract.Kind == "api" && contract.Body != nil {
			if contract.Body.Version != nil {
				status := "stable"
				if contract.Body.Status != nil {
					status = *contract.Body.Status
				}
				versions = append(versions, fmt.Sprintf("- **%s**: %s", *contract.Body.Version, status))
				hasLifecycle = true
			}
		}
	}

	if len(versions) > 0 {
		for _, v := range versions {
			sb.WriteString(v + "\n")
		}
		sb.WriteString("\n")
	}

	if !hasLifecycle {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Migration Guide
	sb.WriteString("### Migration Guide\n\n")
	hasMigration := false

	if migration, ok := arch.MetaString("apiMigrationGuide"); ok {
		sb.WriteString(migration + "\n")
		hasMigration = true
	}

	if !hasMigration {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeMultiRegionConsiderations(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Multi-Region Architecture\n\n")

	// Current Deployment
	sb.WriteString("### Current Deployment\n\n")
	hasDeployment := false

	// Extract from deployment nodes
	regions := []string{}
	for _, deployment := range arch.DeploymentNodes {
		if deployment.Label != "" {
			regions = append(regions, deployment.Label)
			hasDeployment = true
		}
	}

	if len(regions) > 0 {
		if len(regions) == 1 {
			sb.WriteString(fmt.Sprintf("- **Primary Region**: %s\n", regions[0]))
		} else {
			sb.WriteString(fmt.Sprintf("- **Primary Region**: %s\n", regions[0]))
			sb.WriteString(fmt.Sprintf("- **Secondary Region(s)**: %s\n", strings.Join(regions[1:], ", ")))
		}
		sb.WriteString("\n")
	}

	if primary, ok := arch.MetaString("primaryRegion"); ok {
		sb.WriteString(fmt.Sprintf("- **Primary Region**: %s\n", primary))
		hasDeployment = true
	}
	if dr, ok := arch.MetaString("drRegion"); ok {
		sb.WriteString(fmt.Sprintf("- **DR Region**: %s\n", dr))
		hasDeployment = true
	}

	if !hasDeployment {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Data Residency
	sb.WriteString("### Data Residency\n\n")
	hasResidency := false

	if residency, ok := arch.MetaString("dataResidency"); ok {
		sb.WriteString(residency + "\n")
		hasResidency = true
	}
	if euData, ok := arch.MetaString("euDataRegion"); ok {
		sb.WriteString(fmt.Sprintf("- **EU Data**: Stored in %s (GDPR compliance)\n", euData))
		hasResidency = true
	}
	if usData, ok := arch.MetaString("usDataRegion"); ok {
		sb.WriteString(fmt.Sprintf("- **US Data**: Stored in %s\n", usData))
		hasResidency = true
	}

	if !hasResidency {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Latency
	sb.WriteString("### Latency\n\n")
	hasLatency := false

	if latency, ok := arch.MetaString("latency"); ok {
		sb.WriteString(latency + "\n")
		hasLatency = true
	}

	if !hasLatency {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Future Expansion
	sb.WriteString("### Future Expansion\n\n")
	hasExpansion := false

	if expansion, ok := arch.MetaString("futureRegions"); ok {
		sb.WriteString(expansion + "\n")
		hasExpansion = true
	}

	if !hasExpansion {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeDataLifecycleManagement(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Data Lifecycle Management\n\n")

	// Data Retention Policies
	sb.WriteString("### Data Retention Policies\n\n")
	hasRetention := false

	// Extract from policies
	for _, policy := range arch.Policies {
		if policy.Category != nil && strings.Contains(strings.ToLower(*policy.Category), "retention") {
			sb.WriteString(fmt.Sprintf("- %s\n", policy.Description))
			hasRetention = true
		}
		if policy.Category != nil && strings.Contains(strings.ToLower(*policy.Category), "data") {
			if strings.Contains(strings.ToLower(policy.Description), "retain") ||
				strings.Contains(strings.ToLower(policy.Description), "retention") {
				sb.WriteString(fmt.Sprintf("- %s\n", policy.Description))
				hasRetention = true
			}
		}
	}

	// Extract from metadata
	if retention, ok := arch.MetaString("dataRetention"); ok {
		sb.WriteString(fmt.Sprintf("- %s\n", retention))
		hasRetention = true
	}
	if orderRetention, ok := arch.MetaString("orderDataRetention"); ok {
		sb.WriteString(fmt.Sprintf("- **Order Data**: %s\n", orderRetention))
		hasRetention = true
	}
	if userRetention, ok := arch.MetaString("userDataRetention"); ok {
		sb.WriteString(fmt.Sprintf("- **User Data**: %s\n", userRetention))
		hasRetention = true
	}
	if logRetention, ok := arch.MetaString("logRetention"); ok {
		sb.WriteString(fmt.Sprintf("- **Logs**: %s\n", logRetention))
		hasRetention = true
	}

	if !hasRetention {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Data Archival
	sb.WriteString("### Data Archival\n\n")
	hasArchival := false

	if archival, ok := arch.MetaString("dataArchival"); ok {
		sb.WriteString(fmt.Sprintf("- **Strategy**: %s\n", archival))
		hasArchival = true
	}
	if process, ok := arch.MetaString("archivalProcess"); ok {
		sb.WriteString(fmt.Sprintf("- **Process**: %s\n", process))
		hasArchival = true
	}
	if cost, ok := arch.MetaString("archivalCost"); ok {
		sb.WriteString(fmt.Sprintf("- **Cost**: %s\n", cost))
		hasArchival = true
	}

	if !hasArchival {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Data Deletion
	sb.WriteString("### Data Deletion\n\n")
	hasDeletion := false

	// Check for GDPR deletion policy
	for _, policy := range arch.Policies {
		if policy.Category != nil && strings.Contains(strings.ToLower(*policy.Category), "privacy") {
			if strings.Contains(strings.ToLower(policy.Description), "delete") ||
				strings.Contains(strings.ToLower(policy.Description), "deletion") {
				sb.WriteString(fmt.Sprintf("- **GDPR Right to Deletion**: %s\n", policy.Description))
				hasDeletion = true
			}
		}
	}

	if deletion, ok := arch.MetaString("dataDeletion"); ok {
		sb.WriteString(fmt.Sprintf("- **Process**: %s\n", deletion))
		hasDeletion = true
	}
	if deletionSLA, ok := arch.MetaString("deletionSLA"); ok {
		sb.WriteString(fmt.Sprintf("- **SLA**: %s\n", deletionSLA))
		hasDeletion = true
	}

	if !hasDeletion {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}
