// pkg/language/ast_postprocess.go
// Package language provides DSL parsing and AST structures.
package language

// extractMetaEntries removed - MetadataBlock now contains MetaEntry directly

// ============================================================================
// Post-Processing Methods
// ============================================================================

// PostProcess populates convenience fields from parsed items.
//
//nolint:funlen,gocyclo // PostProcess is long and complex
func (a *Architecture) PostProcess() {
	// Count items by type for pre-allocation
	var systemsCount, containersCount, componentsCount, dataStoresCount, queuesCount int
	var personsCount, relationsCount, requirementsCount, adrsCount, sharedArtifactsCount int
	var librariesCount, metadataCount, contractsCount, constraintsCount, conventionsCount int
	var deploymentNodesCount, scenariosCount, policiesCount, flowsCount int

	for i := range a.Items {
		item := &a.Items[i]
		if item.System != nil {
			systemsCount++
		}
		if item.Container != nil {
			containersCount++
		}
		if item.Component != nil {
			componentsCount++
		}
		if item.DataStore != nil {
			dataStoresCount++
		}
		if item.Queue != nil {
			queuesCount++
		}
		if item.Person != nil {
			personsCount++
		}
		if item.Relation != nil {
			relationsCount++
		}
		if item.Requirement != nil {
			requirementsCount++
		}
		if item.ADR != nil {
			adrsCount++
		}
		if item.SharedArtifact != nil {
			sharedArtifactsCount++
		}
		if item.Library != nil {
			librariesCount++
		}
		if item.Metadata != nil {
			metadataCount += len(item.Metadata.Entries)
		}
		if item.ContractsBlock != nil {
			contractsCount += len(item.ContractsBlock.Contracts)
		}
		if item.ConstraintsBlock != nil {
			constraintsCount += len(item.ConstraintsBlock.Entries)
		}
		if item.ConventionsBlock != nil {
			conventionsCount += len(item.ConventionsBlock.Entries)
		}
		if item.DeploymentNode != nil {
			deploymentNodesCount++
		}
		if item.Scenario != nil {
			scenariosCount++
		}
		if item.Policy != nil {
			policiesCount++
		}
		if item.Flow != nil {
			flowsCount++
		}
	}

	// Pre-allocate slices
	a.Systems = make([]*System, 0, systemsCount)
	a.Containers = make([]*Container, 0, containersCount)
	a.Components = make([]*Component, 0, componentsCount)
	a.DataStores = make([]*DataStore, 0, dataStoresCount)
	a.Queues = make([]*Queue, 0, queuesCount)
	a.Persons = make([]*Person, 0, personsCount)
	a.Relations = make([]*Relation, 0, relationsCount)
	a.Requirements = make([]*Requirement, 0, requirementsCount)
	a.ADRs = make([]*ADR, 0, adrsCount)
	a.SharedArtifacts = make([]*SharedArtifact, 0, sharedArtifactsCount)
	a.Libraries = make([]*Library, 0, librariesCount)
	a.Metadata = make([]*MetaEntry, 0, metadataCount)
	a.Contracts = make([]*Contract, 0, contractsCount)
	a.Constraints = make([]*ConstraintEntry, 0, constraintsCount)
	a.Conventions = make([]*ConventionEntry, 0, conventionsCount)
	a.DeploymentNodes = make([]*DeploymentNode, 0, deploymentNodesCount)
	a.Scenarios = make([]*Scenario, 0, scenariosCount)
	a.Policies = make([]*Policy, 0, policiesCount)
	a.Flows = make([]*Flow, 0, flowsCount)

	for i := range a.Items {
		item := &a.Items[i]

		if item.System != nil {
			item.System.PostProcess()
			a.Systems = append(a.Systems, item.System)
		}
		if item.Container != nil {
			item.Container.PostProcess()
			a.Containers = append(a.Containers, item.Container)
		}
		if item.Component != nil {
			item.Component.PostProcess()
			a.Components = append(a.Components, item.Component)
		}
		if item.DataStore != nil {
			item.DataStore.PostProcess()
			a.DataStores = append(a.DataStores, item.DataStore)
		}
		if item.Queue != nil {
			item.Queue.PostProcess()
			a.Queues = append(a.Queues, item.Queue)
		}
		if item.Person != nil {
			item.Person.PostProcess()
			a.Persons = append(a.Persons, item.Person)
		}
		if item.Relation != nil {
			normalizeRelation(item.Relation)
			a.Relations = append(a.Relations, item.Relation)
		}
		if item.Requirement != nil {
			item.Requirement.PostProcess()
			a.Requirements = append(a.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			item.ADR.PostProcess()
			a.ADRs = append(a.ADRs, item.ADR)
		}
		if item.SharedArtifact != nil {
			a.SharedArtifacts = append(a.SharedArtifacts, item.SharedArtifact)
		}
		if item.Library != nil {
			item.Library.PostProcess()
			a.Libraries = append(a.Libraries, item.Library)
		}
		if item.Metadata != nil {
			a.Metadata = append(a.Metadata, item.Metadata.Entries...)
		}
		if item.ContractsBlock != nil {
			a.Contracts = append(a.Contracts, item.ContractsBlock.Contracts...)
		}
		if item.ConstraintsBlock != nil {
			a.Constraints = append(a.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			a.Conventions = append(a.Conventions, item.ConventionsBlock.Entries...)
		}
		if item.DeploymentNode != nil {
			item.DeploymentNode.PostProcess()
			a.DeploymentNodes = append(a.DeploymentNodes, item.DeploymentNode)
		}
		if item.Scenario != nil {
			item.Scenario.PostProcess()
			a.Scenarios = append(a.Scenarios, item.Scenario)
		}
		if item.Policy != nil {
			item.Policy.PostProcess()
			a.Policies = append(a.Policies, item.Policy)
		}
		if item.Flow != nil {
			item.Flow.PostProcess()
			a.Flows = append(a.Flows, item.Flow)
		}
		if item.Views != nil {
			a.Views = item.Views
		}
		if item.Properties != nil {
			if a.Properties == nil {
				// Pre-allocate with estimated capacity
				a.Properties = make(map[string]string, len(item.Properties.Entries))
			}
			for _, entry := range item.Properties.Entries {
				a.Properties[entry.Key] = entry.Value
			}
		}
		if item.Style != nil {
			if a.Style == nil {
				a.Style = make(map[string]string)
			}
			for _, entry := range item.Style.Entries {
				a.Style[entry.Key] = entry.Value
			}
		}
		if item.Description != nil {
			a.Description = item.Description
		}
		// View type removed - not in simplified plan
	}

	a.inferImpliedRelationships()
}
