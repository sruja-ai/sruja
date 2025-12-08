# Sruja Strategic Roadmap: The Path to v1.0

This roadmap outlines the strategic initiatives to transform Sruja into a comprehensive Architecture-as-Code platform. Our vision for v1.0 is to move beyond static diagrams to **active governance**, **production enforcement**, and **seamless integration** with the DevOps ecosystem.

## Theme 1: Advanced Governance & Compliance
*Goal: Codify architectural decisions and enforce them automatically.*

### 1.1 Policy as Code (Rego Integration)
- **Objective**: Allow defining complex architectural policies using standard policy languages.
- **Features**:
    - **Rego Integration**: Integrate OPA (Open Policy Agent) to allow writing rules in Rego against the Sruja JSON export.
    - **Native Policy Engine**: Enhance Sruja's native `policy` block to support more granular assertions (e.g., `allow if ...`, `deny if ...`).
    - **Compliance Packs**: Pre-built policy libraries for GDPR, SOC2, HIPAA, and ISO27001.

### 1.2 Architectural Guardrails
- **Objective**: Prevent architectural drift and "big ball of mud" systems.
- **Features**:
    - **Layer Enforcement**: Strict enforcement of layered architectures (e.g., "Presentation cannot access Data directly").
    - **Dependency Rules**: Allow/deny lists for package and component dependencies.
    - **Drift Detection**: Compare the defined architecture against the actual implementation (via code analysis or runtime tracing) and report discrepancies.

## Theme 2: Production Reality & Data Flow
*Goal: Bridge the gap between design and runtime reality.*

### 2.1 Production Data Flow Enforcement
- **Objective**: Ensure that data flows in production match the architectural design.
- **Features**:
    - **Service Mesh Integration**: Generate Istio/Linkerd configuration from Sruja `flow` definitions to enforce traffic rules.
    - **API Gateway Config**: Generate Kong/Apigee configuration to enforce API contracts defined in Sruja.
    - **Data Lineage**: Visualize and validate data lineage from source to sink, flagging potential leaks or unauthorized access.

### 2.2 Runtime Verification
- **Objective**: Verify that the system behaves as designed.
- **Features**:
    - **Chaos Engineering**: Generate chaos experiments (e.g., using Chaos Mesh) based on Sruja's failure scenarios.
    - **Contract Testing**: Generate contract tests (e.g., Pact) from component interactions to verify compatibility in CI/CD.

## Theme 3: Extensibility & Ecosystem
*Goal: Make Sruja adaptable to any environment or workflow.*

### 3.1 Plugin System
- **Objective**: Allow the community to extend Sruja without modifying the core.
- **Features**:
    - **WASM Plugins**: Support WebAssembly plugins for custom validators, generators, and linters. This allows plugins written in Rust, Go, TS, etc.
    - **Custom Rules**: Allow defining custom validation rules in Sruja DSL or via external scripts.

### 3.2 DevOps Integration (Terraform/OpenTofu)
- **Objective**: Seamlessly integrate architecture definition with infrastructure provisioning.
- **Features**:
    - **IaC Generation**: Generate Terraform/OpenTofu HCL from Sruja `deployment` nodes.
    - **Two-Way Sync**: Import existing Terraform state to reverse-engineer the architecture model.
    - **CI/CD Integration**: GitHub Actions and GitLab CI runners to block PRs that violate architectural policies.

## Theme 4: The "Rust-like" Compiler Experience (Completed/Ongoing)
*Goal: Make the compiler a pair programmer.*

- **Rich Error Reporting**: Context-aware errors with suggestions (Completed).
- **LSP Improvements**: "Quick Fixes", auto-imports, and smart autocomplete (Ongoing).
- **`sruja init`**: Project scaffolding (Completed).

## Roadmap Timeline

### Phase 1: Foundation (Current)
- [x] Rich Error Reporting
- [x] Basic Linter Rules (Cycles, Orphans, Layers)
- [x] `sruja init`
- [ ] VS Code Extension Polish

### Phase 2: Governance & Extensibility (Next)
- [ ] Rego/OPA Integration
- [ ] WASM Plugin System
- [ ] Terraform Generator (Alpha)
- [ ] Compliance Library (`lib/compliance`)

### Phase 3: Production Enforcement (v1.0 Goal)
- [ ] Drift Detection (Code vs. Design)
- [ ] Service Mesh / API Gateway Generation
- [ ] Data Lineage Visualization
- [ ] Full CI/CD Integration Suite
