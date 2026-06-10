---
title: "Adoption Guide"
weight: 21
summary: "Complete guide to evaluating and adopting Sruja for your organization."
---

# Sruja Adoption Guide

## Using Sruja in your repo

For a short, practical guide to the new product shape, see **[Core vs extensions](using-sruja-in-your-project.md)**. The rest of this adoption guide helps you evaluate fit and plan rollout.

## Canonical pilot path (recommended)

Use a single, repeatable workflow to evaluate Sruja. Start with the core loop first:

```bash
curl -fsSL https://sruja.ai/install.sh | bash
sruja start -r .
sruja drift -r . --structural-only --advisory
sruja focus -r . --file path/to/file.rs
sruja verify-task --profile coding -r .
```

Only after that should you decide whether to add reviewed intent in Git:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
sruja lint repo.sruja
sruja sync -r .
sruja drift -r . -a repo.sruja
```

This keeps evaluation honest: prove the core loop first, then layer on richer authoring and governance only when needed.

## Is Sruja Right for Your Organization?

### Quick Self-Assessment

Answer these questions to determine if Sruja addresses your needs:

#### Architecture & Documentation Pain Points

- [ ] Do your architecture diagrams become outdated within weeks?
- [ ] Do engineers spend significant time maintaining documentation?
- [ ] Is there confusion about "the latest architecture diagram"?
- [ ] Do new engineers struggle to understand system architecture?
- [ ] Are architectural decisions lost when senior engineers leave?

**If 3+ are "Yes"** → Sruja can help

#### Compliance & Governance Needs

- [ ] Do you need to comply with regulations (HIPAA, SOC2, PCI-DSS, GDPR)?
- [ ] Are compliance audits time-consuming and risky?
- [ ] Do you struggle to prove architectural controls meet requirements?
- [ ] Are security policies documented but not enforced?
- [ ] Do you need to demonstrate compliance to auditors?

**If 2+ are "Yes"** → Sruja's policy-as-code is valuable

#### Technical Architecture Challenges

- [ ] Do you have microservices that need explicit guardrails?
- [ ] Are you experiencing architectural drift (implementation vs. design)?
- [ ] Do you need to enforce service boundaries and dependencies?
- [ ] Are circular dependencies causing issues?
- [ ] Do you need to generate infrastructure from architecture?

**If 2+ are "Yes"** → Sruja's validation and enforcement help

#### DevOps & Engineering Culture

- [ ] Do you use Git/GitOps workflows?
- [ ] Do you have CI/CD pipelines?
- [ ] Do you value "everything as code" (IaC, GitOps)?
- [ ] Do you want architecture changes in PR reviews?
- [ ] Do you need architecture to integrate with Terraform/Istio/etc.?

**If 3+ are "Yes"** → Sruja fits your workflow

## Organization Size & Maturity

**Sruja is ideal for:**

- ✅ **Startups (10-50 engineers)**: Fast scaling, need consistency
- ✅ **Scale-ups (50-200 engineers)**: Managing complexity, compliance needs
- ✅ **Enterprises (200+ engineers)**: Governance, compliance, knowledge management

**Sruja may not be ideal if:**

- ❌ You have < 5 engineers (overhead may outweigh benefits)
- ❌ You don't use version control or CI/CD
- ❌ You prefer visual-only tools (no code/DSL)
- ❌ You have no need for shared architecture context, CI checks, or policy guardrails

## Decision Framework

### Step 1: Define Your Goals

**What problem are you trying to solve?**

| Goal                              | Sruja Benefit                            | Priority |
| --------------------------------- | ---------------------------------------- | -------- |
| **Reduce documentation overhead** | Architecture-as-code stays current       | High     |
| **Ensure compliance**             | Policy-as-code with automated validation | High     |
| **Prevent architectural drift**   | Automated validation in CI/CD            | Medium   |
| **Faster onboarding**             | Living documentation in codebase         | Medium   |
| **Enforce service boundaries**    | Layer and dependency validation          | Medium   |
| **Generate infrastructure**       | Terraform/OpenTofu generation (roadmap)  | Low      |

**Action**: Rank your top 3 goals. Sruja should address at least 2.

### Step 2: Calculate Value & ROI

**Note**: Sruja is free and open source. This ROI calculation measures **time savings and value**, not purchase cost.

**Quick Value Calculator:**

```
Time Savings = (Engineers × Hours/Week × 0.7) × 50 weeks × $100/hour
Onboarding Savings = (New Engineers/Year × 2 weeks × 0.5) × $150k/year ÷ 50
Risk Reduction = Compliance Failures Avoided × $100k

Total Value = Time Savings + Onboarding + Risk Reduction
```

**Example** (10 senior engineers, 20 new engineers/year):

- Time: 10 × 4 hours × 0.7 × 50 × $100 = **$140k/year**
- Onboarding: 20 × 2 × 0.5 × $150k ÷ 50 = **$60k/year**
- Risk: 1 failure avoided = **$100k** (one-time)
- **Total Value: $200k+ per year**

**ROI**: Since Sruja is free, ROI is essentially infinite - you get value with zero cost.

### Step 3: Assess Technical Fit

**Evaluate your technical stack:**

| Technology                            | Sruja Integration              | Status               |
| ------------------------------------- | ------------------------------ | -------------------- |
| **Git/GitHub/GitLab**                 | Native integration             | ✅ Available         |
| **CI/CD (GitHub Actions, GitLab CI)** | Validation in pipelines        | ✅ Available         |
| **Terraform/OpenTofu**                | Infrastructure generation      | 🚧 Roadmap (Medium-Term) |
| **Kubernetes/Istio**                  | Service mesh config generation | 🚧 Roadmap (Long-Term) |
| **API Gateways (Kong, Apigee)**       | Config generation              | 🚧 Roadmap (Long-Term) |
| **OPA (Open Policy Agent)**           | Policy integration             | 🚧 Roadmap (Medium-Term) |

**Action**:

- If you need Git/CI/CD integration → ✅ Ready now
- If you need Terraform/Istio/OPA → 🚧 On roadmap (see [Roadmap Discussions](https://github.com/sruja-ai/sruja/discussions)) — you can pilot with current features now

## Evaluation Process

### Phase 1: Discovery (Week 1)

**Activities:**

1. Review Sruja documentation
2. Install CLI: `curl -fsSL https://sruja.ai/install.sh | bash`
3. Run the core loop: `sruja start -r .`, `sruja drift -r . --structural-only --advisory`, `sruja focus -r . --file <path>`, `sruja verify-task --profile coding -r .`
4. If the team needs reviewed intent, author `repo.sruja` with the skill, then validate and refresh evidence: `sruja lint repo.sruja` then `sruja sync -r .`
5. Capture the first alignment signal: `sruja drift -r . -a repo.sruja` when reviewed intent exists
6. Install VS Code extension for syntax highlighting and diagnostics if the team will edit `.sruja` files

**Deliverable**: Understanding of Sruja capabilities

### Phase 2: Proof of Concept (Weeks 2-4)

**Activities:**

1. Expand `repo.sruja` to cover the repo boundary and core containers that matter for reviews
2. Integrate validation and drift into CI/CD (`sruja lint repo.sruja` and `sruja drift -r . -a repo.sruja`)
3. Establish a PR review rule: architecture changes ship with code changes (same diff)
4. Measure signal vs noise (how many findings you keep vs ignore)

**Success Criteria:**

- [ ] Can model systems accurately
- [ ] Validation catches real issues
- [ ] Team sees value
- [ ] Time savings measurable

**Deliverable**: PoC report with value estimate

### Phase 3: Pilot (Months 2-3)

**Activities:**

1. Roll out to 1-2 teams
2. Establish best practices
3. Create internal documentation
4. Measure compliance improvements

**Success Criteria:**

- [ ] Architecture stays current
- [ ] Compliance validation working
- [ ] Team adoption > 80%
- [ ] Positive value demonstrated

**Deliverable**: Pilot report with go/no-go recommendation

## Decision Checklist

### Must-Have Requirements

- [ ] **Problem Fit**: Sruja addresses 2+ of your top goals
- [ ] **Value Positive**: Calculated value > $100k/year (or equivalent time savings)
- [ ] **Technical Fit**: Git/CI/CD integration available (or roadmap acceptable)
- [ ] **Team Readiness**: Team comfortable with code-based tools
- [ ] **Leadership Support**: Time allocated for adoption (no budget needed - Sruja is free)

### Nice-to-Have Requirements

- [ ] Advanced features needed (Terraform, Istio, OPA)
- [ ] Compliance requirements (HIPAA, SOC2, PCI-DSS)
- [ ] Large team (100+ engineers)
- [ ] Microservices architecture

### Decision Matrix

| Criteria           | Weight | Your Score (1-5) | Weighted Score |
| ------------------ | ------ | ---------------- | -------------- |
| Problem fit        | 30%    | \_\_\_           | \_\_\_         |
| Value/ROI          | 25%    | \_\_\_           | \_\_\_         |
| Technical fit      | 20%    | \_\_\_           | \_\_\_         |
| Team readiness     | 15%    | \_\_\_           | \_\_\_         |
| Leadership support | 10%    | \_\_\_           | \_\_\_         |
| **Total**          | 100%   |                  | **\_\_\_/5.0** |

**Decision Rule**:

- **> 4.0**: Strong fit → Proceed with pilot
- **3.5-4.0**: Good fit → Consider pilot
- **< 3.5**: Weak fit → Reassess or wait

## Common Concerns & Objections

### "We already have architecture documentation"

**Response**: Sruja doesn't replace documentation — it makes it **executable**. Your documentation becomes code that:

- Stays current (version-controlled)
- Validates automatically
- Enforces policies
- Integrates with DevOps

### "Our team isn't technical enough for a DSL"

**Response**: Sruja's DSL is designed for **all developers**:

- 1st-year CS students productive in 10 minutes
- Progressive disclosure (simple → advanced)
- Rich error messages guide users
- VS Code extension with full LSP support (autocomplete, go-to-definition, rename, find references, and more) - see [VS Code Extension Guide](vscode)

### "We don't have compliance requirements"

**Response**: Sruja provides value beyond compliance:

- Faster onboarding (50% reduction)
- Reduced documentation time (20-30%)
- Architectural validation (prevents drift)
- Knowledge preservation

### "The roadmap features we need aren't ready"

**Response**:

- Core features (validation, CI/CD) are **available now**
- Roadmap features (Terraform, Istio, OPA) are planned for **Medium-Term to Long-Term** (see [Roadmap Discussions](https://github.com/sruja-ai/sruja/discussions))
- You can start with core features and add advanced later
- Early adoption gives you influence on roadmap priorities

## Success Metrics

### Track These KPIs

| Metric                          | Baseline        | Target (3 months)     | Target (6 months)     |
| ------------------------------- | --------------- | --------------------- | --------------------- |
| **Documentation time**          | X hours/week    | X × 0.7 hours/week    | X × 0.5 hours/week    |
| **Onboarding time**             | X weeks         | X × 0.7 weeks         | X × 0.5 weeks         |
| **Architecture freshness**      | X% outdated     | < 10% outdated        | < 5% outdated         |
| **Compliance violations**       | X per quarter   | X × 0.5 per quarter   | 0 per quarter         |
| **Architectural issues caught** | X in production | X × 0.3 in production | X × 0.1 in production |

## Next Steps

### Immediate Actions

1. **Complete Self-Assessment** (above)
2. **Calculate Value** (Step 2)
3. **Try Sruja** (see [Getting Started](getting-started.md))
4. **Join Community** (GitHub Discussions)

### Decision Timeline

- **Week 1**: Self-assessment and value calculation
- **Week 2-4**: Proof of concept
- **Month 2-3**: Pilot program
- **Month 4+**: Full rollout (if successful)

## Resources

- **Getting Started**: [Getting Started Guide](getting-started.md)
- **Executive Overview**: [Executive Overview](/investors/executive-overview)
- **Adoption Playbook**: [Adoption Playbook](adoption-playbook.md)
- **Decision Framework**: [Quick Decision Framework](docs/decision-framework)

## Open Source & Community Support

Sruja is **free and open source** (Apache 2.0 licensed), developed by and for the community. You can:

- **Use it freely**: No licensing fees or restrictions
- **Contribute**: Submit PRs, report issues, suggest features
- **Extend it**: Build custom validators, exporters, and integrations
- **Join the community**: Participate in [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions), share use cases, and learn from others

## Professional Services

While Sruja is open source and free to use, professional consulting services are available for organizations that need:

- **Implementation support**: Help rolling out Sruja across teams and systems
- **Context workflow guidance**: Establish evidence, validation, and review patterns for AI-assisted architecture work
- **Custom integrations**: Integrate Sruja with existing CI/CD, infrastructure, and monitoring tools
- **Training**: Team training on Sruja DSL, validation patterns, and architectural modeling
- **Custom development**: Build custom validators, exporters, or platform integrations

Contact the team through [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) to discuss your needs.

## Future Platform Vision

Sruja is designed to evolve as a context engineering platform for AI-assisted architecture work:

- **Live System Review**: Compare actual runtime behavior against reviewed architecture truth to detect drift and violations.
- **Gap Analysis**: Identify missing components, undocumented dependencies, and architectural gaps from evidence.
- **Continuous Validation**: Keep repo context fresh as code, docs, and deployment artifacts change.
- **Review Support**: Help humans and AI agents evaluate whether proposed changes still match intent.

These capabilities are planned for future releases. The current open source foundation provides the evidence, validation, and context layers for this evolution, and community feedback helps shape the roadmap.

---

**Note**: This guide helps you evaluate whether Sruja is the right fit for your organization and how to adopt it successfully.

**Ready to evaluate Sruja? Start with the Self-Assessment above.**
