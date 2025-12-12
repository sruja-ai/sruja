---
title: "Adoption Guide"
weight: 21
summary: "Complete guide to evaluating and adopting Sruja for your organization."
---

# Sruja Adoption Guide

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

- [ ] Do you have microservices that need governance?
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
- ❌ You have no compliance or governance requirements

## Decision Framework

### Step 1: Define Your Goals

**What problem are you trying to solve?**

| Goal | Sruja Benefit | Priority |
|------|---------------|----------|
| **Reduce documentation overhead** | Architecture-as-code stays current | High |
| **Ensure compliance** | Policy-as-code with automated validation | High |
| **Prevent architectural drift** | Automated validation in CI/CD | Medium |
| **Faster onboarding** | Living documentation in codebase | Medium |
| **Enforce service boundaries** | Layer and dependency validation | Medium |
| **Generate infrastructure** | Terraform/OpenTofu generation (roadmap) | Low |

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

| Technology | Sruja Integration | Status |
|------------|-------------------|--------|
| **Git/GitHub/GitLab** | Native integration | ✅ Available |
| **CI/CD (GitHub Actions, GitLab CI)** | Validation in pipelines | ✅ Available |
| **Terraform/OpenTofu** | Infrastructure generation | 🚧 Roadmap (Phase 2) |
| **Kubernetes/Istio** | Service mesh config generation | 🚧 Roadmap (Phase 3) |
| **API Gateways (Kong, Apigee)** | Config generation | 🚧 Roadmap (Phase 3) |
| **OPA (Open Policy Agent)** | Policy integration | 🚧 Roadmap (Phase 2) |

**Action**: 
- If you need Git/CI/CD integration → ✅ Ready now
- If you need Terraform/Istio/OPA → 🚧 On roadmap (see [Roadmap Discussions](https://github.com/sruja-ai/sruja/discussions)) — you can pilot with current features now

## Evaluation Process

### Phase 1: Discovery (Week 1)

**Activities:**
1. Review Sruja documentation
2. Try the online playground
3. Install CLI: `curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash`
4. Model a simple existing system

**Deliverable**: Understanding of Sruja capabilities

### Phase 2: Proof of Concept (Weeks 2-4)

**Activities:**
1. Model 1-2 real systems in Sruja
2. Integrate validation into CI/CD
3. Document architecture decisions as ADRs
4. Measure time savings

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

| Criteria | Weight | Your Score (1-5) | Weighted Score |
|----------|--------|------------------|----------------|
| Problem fit | 30% | ___ | ___ |
| Value/ROI | 25% | ___ | ___ |
| Technical fit | 20% | ___ | ___ |
| Team readiness | 15% | ___ | ___ |
| Leadership support | 10% | ___ | ___ |
| **Total** | 100% | | **___/5.0** |

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
- VS Code extension with autocomplete

### "We don't have compliance requirements"

**Response**: Sruja provides value beyond compliance:
- Faster onboarding (50% reduction)
- Reduced documentation time (20-30%)
- Architectural validation (prevents drift)
- Knowledge preservation

### "The roadmap features we need aren't ready"

**Response**: 
- Core features (validation, CI/CD) are **available now**
- Roadmap features (Terraform, Istio, OPA) are planned for **Phase 2-3** (see [Roadmap Discussions](https://github.com/sruja-ai/sruja/discussions))
- You can start with core features and add advanced later
- Early adoption gives you influence on roadmap priorities

## Success Metrics

### Track These KPIs

| Metric | Baseline | Target (3 months) | Target (6 months) |
|--------|----------|-------------------|-------------------|
| **Documentation time** | X hours/week | X × 0.7 hours/week | X × 0.5 hours/week |
| **Onboarding time** | X weeks | X × 0.7 weeks | X × 0.5 weeks |
| **Architecture freshness** | X% outdated | < 10% outdated | < 5% outdated |
| **Compliance violations** | X per quarter | X × 0.5 per quarter | 0 per quarter |
| **Architectural issues caught** | X in production | X × 0.3 in production | X × 0.1 in production |

## Next Steps

### Immediate Actions

1. **Complete Self-Assessment** (above)
2. **Calculate Value** (Step 2)
3. **Try Sruja** (see [Getting Started](/docs/getting-started))
4. **Join Community** (GitHub Discussions, Discord, etc.)

### Decision Timeline

- **Week 1**: Self-assessment and value calculation
- **Week 2-4**: Proof of concept
- **Month 2-3**: Pilot program
- **Month 4+**: Full rollout (if successful)

## Resources

- **Getting Started**: [Getting Started Guide](/docs/getting-started)
- **Executive Overview**: [Executive Overview](/investors/executive-overview)
- **Adoption Playbook**: [Adoption Playbook](/docs/adoption-playbook)
- **Decision Framework**: [Quick Decision Framework](/docs/decision-framework)

## Open Source & Community Support

Sruja is **free and open source** (MIT licensed), developed by and for the community. You can:

- **Use it freely**: No licensing fees or restrictions
- **Contribute**: Submit PRs, report issues, suggest features
- **Extend it**: Build custom validators, exporters, and integrations
- **Join the community**: Participate in [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions), share use cases, and learn from others

## Professional Services

While Sruja is open source and free to use, professional consulting services are available for organizations that need:

- **Implementation support**: Help rolling out Sruja across teams and systems
- **Best practices guidance**: Establish architectural governance patterns and workflows
- **Custom integrations**: Integrate Sruja with existing CI/CD, infrastructure, and monitoring tools
- **Training**: Team training on Sruja DSL, validation patterns, and architectural modeling
- **Custom development**: Build custom validators, exporters, or platform integrations

Contact the team through [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) to discuss your needs.

## Future Platform Vision

Sruja is designed to evolve into a comprehensive platform for architectural governance:

- **Live System Review**: Compare actual runtime behavior against architectural models to detect drift and violations.
- **Gap Analysis**: Automatically identify missing components, undocumented dependencies, and architectural gaps.
- **Continuous Validation**: Monitor production systems against architectural policies and constraints in real-time.
- **Compliance Monitoring**: Track and report on architectural compliance across services and deployments.

These capabilities are planned for future releases. The current open source foundation provides the building blocks for this evolution, and community feedback helps shape the roadmap.

---

**Note**: This guide helps you evaluate whether Sruja is the right fit for your organization and how to adopt it successfully.

**Ready to evaluate Sruja? Start with the Self-Assessment above.**
