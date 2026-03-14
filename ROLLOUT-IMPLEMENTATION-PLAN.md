# Sruja Rollout Implementation Plan

## Executive Summary
Move Sruja from "strong pilot candidate" to "safe to adopt as the daily architecture truth layer" through phased implementation with measurable success criteria.

---

## Success Metrics & Thresholds

### Baseline Measurements (Pre-Pilot)
| Metric | Baseline Target | Pilot Success Threshold | Rollout Gate |
|--------|-----------------|------------------------|--------------|
| PR drift noise rate | Establish baseline | <15% of PRs with irrelevant drift | <10% |
| Time to refresh context | Establish baseline | <5 minutes | <3 minutes |
| Useful signal on arch-changing PRs | Establish baseline | >60% | >75% |
| Editor task accuracy (with Sruja) | Measure control group | >15% improvement | >25% improvement |
| Teams retaining Sruja after pilot | N/A | 100% retention | 100% retention |
| DSL freshness via PR review | Establish baseline | >80% via PR review | >90% via PR review |

### Rollback Criteria
Any of the following triggers immediate pilot pause:
- PR drift noise rate >30%
- <50% of arch-changing PRs receive useful signal
- >1 pilot team drops Sruja before pilot end
- Teams report Sruja adds >30 minutes/week overhead

---

## Phase 0: Baseline Measurement (Week 1-2)

### Goals
- Establish current-state metrics for comparison
- Set up measurement infrastructure

### Tasks
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Instrument PR analysis for noise measurement | Backend | 3 days | None |
| Set up editor accuracy A/B test framework | Platform | 3 days | None |
| Create pilot team onboarding survey | PM | 1 day | None |
| Document current context-refresh workflow timing | PM | 2 days | None |

### Deliverables
- Metrics dashboard with baseline values
- Pilot team selection confirmed (2-3 teams)
- Pre-pilot survey completed

---

## Phase 1: Fix Product Blockers (Week 3-6)

### 1.1 Multi-Repo Composition
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Update `compose` to accept `*.repo.bundle.json` patterns | Backend | 2 days | None |
| Add recursive bundle discovery | Backend | 2 days | 1.1.1 |
| Implement duplicate bundle ID detection | Backend | 1 day | 1.1.2 |
| Add conflict detection with explicit messages | Backend | 2 days | 1.1.3 |
| Document accepted bundle patterns | Docs | 1 day | 1.1.4 |
| Add contract tests for compose flow | QA | 2 days | 1.1.4 |

### 1.2 Low-Noise Drift Detection
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Create shared scan-scope resolver module | Backend | 2 days | None |
| Add default exclusions (docs, tests, generated, book) | Backend | 1 day | 1.2.1 |
| Update `check` to use scoped resolver | Backend | 2 days | 1.2.2 |
| Update `review` to use scoped resolver | Backend | 2 days | 1.2.2 |
| Consolidate drift categories (target: <5 categories) | Backend | 2 days | 1.2.3, 1.2.4 |
| Add JSON output contract tests | QA | 1 day | 1.2.5 |

### 1.3 User Journey Alignment
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Update skill manifest to prefer `repo.sruja` | Platform | 1 day | None |
| Update install/getting-started docs | Docs | 1 day | 1.3.1 |
| Update extension copy and onboarding | Platform | 1 day | 1.3.1 |
| Update examples and scripts | Docs | 1 day | 1.3.1 |
| Add `architecture.sruja` deprecation notice | Backend | 0.5 days | 1.3.1 |

### Phase 1 Exit Criteria
- [ ] `cargo test --workspace` passes
- [ ] Extension build green
- [ ] Multi-bundle compose works with directory input
- [ ] `check -r .` on Sruja repo produces <10 lines of relevant output
- [ ] All surfaced paths reference `repo.sruja`

---

## Phase 2: Daily Loop Usability (Week 7-8)

### 2.1 Standardized Workflow
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Define `sync` → `status` → `review` → `check` contract | Backend | 1 day | Phase 1 |
| Add `status` command with truth state output | Backend | 2 days | 2.1.1 |
| Add freshness metadata to all commands | Backend | 1 day | 2.1.2 |
| Create machine-friendly output contract | Backend | 1 day | 2.1.3 |

### 2.2 Review-First Drift
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Implement change-impact filtering | Backend | 2 days | Phase 1 |
| Add actionable suggestion mapping | Backend | 2 days | 2.2.1 |
| Add regression tests for trivial-change filtering | QA | 1 day | 2.2.2 |

### 2.3 Output Contract Standardization
```json
{
  "truth_status": "current|drifted|unknown",
  "baseline": "repo.sruja",
  "violations_count": 0,
  "health_score": 0.95,
  "scan_scope": {
    "included": ["src/**/*.rs"],
    "excluded": ["docs/**", "tests/**"]
  },
  "updated_at": "2026-03-14T10:00:00Z",
  "freshness_hours": 2
}
```

| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Implement JSON output for `check` | Backend | 1 day | 2.1.4 |
| Implement JSON output for `review` | Backend | 1 day | 2.1.4 |
| Implement JSON output for `status` | Backend | 1 day | 2.1.4 |
| Add GitHub Actions output format | Backend | 1 day | 2.3.1 |

### Phase 2 Exit Criteria
- [ ] `sync` → `status` → `review` → `check` workflow documented
- [ ] All commands support JSON output
- [ ] Trivial code changes produce no drift output
- [ ] Output contract tests pass

---

## Phase 3: Multi-Repo Proof (Week 9-11)

### 3.1 Pilot Repo Setup
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Select 3 pilot repos with dependencies | PM | 1 day | Phase 2 |
| Configure API contract dependency between repos | Platform | 2 days | 3.1.1 |
| Configure shared package dependency | Platform | 1 day | 3.1.1 |
| Document external system representation | Platform | 1 day | 3.1.1 |

### 3.2 Federation Validation
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Publish bundle from each pilot repo | Backend | 1 day | 3.1 |
| Compose system index from bundles | Backend | 1 day | 3.2.1 |
| Validate cross-repo conflict detection | QA | 2 days | 3.2.2 |
| Test editor retrieval of impacted slice | Platform | 2 days | 3.2.2 |

### 3.3 System Graph Ownership
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Define ownership model document | PM | 1 day | None |
| Implement ownership metadata in bundles | Backend | 2 days | 3.3.1 |
| Add conflict resolution UI/messaging | Backend | 2 days | 3.3.2 |
| Document ownership resolution process | Docs | 1 day | 3.3.3 |

### Phase 3 Exit Criteria
- [ ] 3 pilot repos publishing bundles
- [ ] System index composes without errors
- [ ] Cross-repo conflicts detected and explained
- [ ] Editor loads impacted slice (not full graph)

---

## Phase 4: Bundle Schema Versioning (Week 12)

| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Define `repo.bundle.json` schema versioning strategy | Backend | 1 day | Phase 3 |
| Add schema version field to bundle format | Backend | 1 day | 4.1 |
| Implement backward-compatible reader | Backend | 2 days | 4.2 |
| Add version mismatch warnings | Backend | 1 day | 4.3 |
| Document versioning policy | Docs | 1 day | 4.4 |

### Versioning Strategy
- Schema version in `schema_version` field (semver)
- Major version bump = breaking changes
- Minor/patch = backward compatible
- Reader supports N-2 major versions
- Mismatch produces warning, not error (for pilot phase)

---

## Phase 5: Pilot Execution (Week 13-18)

### 5.1 Pilot Onboarding (Week 13)
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Conduct pilot team training | PM | 2 days | Phase 4 |
| Set up pilot repo integrations | Platform | 2 days | 5.1.1 |
| Configure CI `check` for pilot repos | Platform | 1 day | 5.1.2 |
| Enable editor context for pilot teams | Platform | 1 day | 5.1.2 |

### 5.2 Pilot Monitoring (Week 14-17)
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Weekly metrics collection | PM | Ongoing | 5.1 |
| Weekly pilot team check-ins | PM | Ongoing | 5.1 |
| Bi-weekly drift noise analysis | Backend | Ongoing | 5.1 |
| Editor accuracy A/B test monitoring | Platform | Ongoing | 5.1 |

### 5.3 Pilot Evaluation (Week 18)
| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Compile pilot metrics report | PM | 2 days | 5.2 |
| Conduct developer interviews | PM | 2 days | 5.2 |
| Conduct tech lead interviews | PM | 1 day | 5.2 |
| Conduct stakeholder interview | PM | 1 day | 5.2 |
| Produce go/no-go recommendation | PM | 1 day | 5.3.1-4 |

### Pilot Success Criteria
- [ ] PR drift noise rate <15%
- [ ] Time to refresh context <5 minutes
- [ ] >60% of arch-changing PRs receive useful signal
- [ ] >15% improvement in editor task accuracy
- [ ] 100% pilot team retention
- [ ] Teams confirm Sruja saves time
- [ ] Teams confirm Sruja doesn't feel like extra process

---

## Phase 6: Broad Rollout Decision (Week 19)

### Decision Gate Checklist
- [ ] `cargo test --workspace` and extension build green
- [ ] Multi-bundle compose works with realistic directory flow
- [ ] `check -r .` low-noise on Sruja repo and pilot repos
- [ ] All surfaced user journeys prefer `repo.sruja`
- [ ] 2+ pilot teams used Sruja for 4+ weeks without dropping
- [ ] Editor-assisted tasks measurably better with Sruja context
- [ ] System-level views consistent with repo-level truth
- [ ] All success metrics meet thresholds

### Rollout Decision Matrix

| Scenario | Action |
|----------|--------|
| All gates pass | Proceed to broad rollout |
| 1-2 gates fail (minor) | 2-week extension, re-evaluate |
| 1-2 gates fail (major) | Return to Phase 1/2 |
| 3+ gates fail | Re-evaluate product-market fit |
| Rollback criteria triggered | Immediate pilot pause, root cause analysis |

---

## Rollback Plan

### Trigger Conditions
- PR drift noise rate >30%
- <50% useful signal on arch-changing PRs
- Any pilot team drops Sruja
- Teams report >30 min/week overhead

### Rollback Steps
1. **Immediate**: Disable CI `check` in pilot repos
2. **Day 1**: Communicate pause to pilot teams
3. **Day 2-3**: Collect structured feedback on issues
4. **Week 1**: Root cause analysis
5. **Week 2**: Decision: fix and re-pilot, or major pivot

### Rollback Communication Template
```
Subject: Sruja Pilot Pause

Team,

We're pausing the Sruja pilot due to [specific trigger].
Next steps: [root cause timeline]
Your feedback: [feedback collection method]

We'll share findings by [date].
```

---

## Test Plan

### Contract Tests
| Test | Command | Expected |
|------|---------|----------|
| Bundle generation | `sruja publish` | Valid `repo.bundle.json` |
| Directory compose | `sruja compose ./bundles/` | System index created |
| Check JSON output | `sruja check -r . --format json` | Valid JSON with contract fields |
| Review JSON output | `sruja review --format json` | Valid JSON with drift items |
| GitHub Actions output | `sruja check --format github` | Valid GitHub Actions annotations |

### Regression Tests
| Test | Expected |
|------|----------|
| Non-production assets excluded | `docs/`, `tests/`, `book/` not in drift |
| Skill manifests reference `repo.sruja` | No `architecture.sruja` references |
| Multi-bundle compose | Multiple bundles in one directory succeed |
| Trivial change no drift | Comment/formatting changes produce no output |
| Schema version mismatch | Warning, not error |

### Pilot Evaluation Tests
| Scenario | Expected Output |
|----------|-----------------|
| Repo-local change, no arch impact | No drift output |
| Repo-local change, arch impact | Actionable drift suggestion |
| Cross-repo contract change | Cross-repo dependency flagged |
| Missing system truth | `unknown` status, not confident guess |
| Conflicting system truth | Explicit conflict message |

---

## Timeline Summary

```
Week 1-2:   Phase 0 - Baseline Measurement
Week 3-6:   Phase 1 - Fix Product Blockers
Week 7-8:   Phase 2 - Daily Loop Usability
Week 9-11:  Phase 3 - Multi-Repo Proof
Week 12:    Phase 4 - Bundle Schema Versioning
Week 13-18: Phase 5 - Pilot Execution
Week 19:    Phase 6 - Rollout Decision
```

**Total Duration: 19 weeks (~5 months)**

---

## Assumptions
- `repo.sruja` is the preferred local truth file
- `architecture.sruja` remains temporarily supported with deprecation notice
- CLI remains deterministic; AI skills orchestrate but don't become truth
- Editors are the main daily surface; stakeholder views derived from composed model
- 2-3 pilot teams available with realistic multi-repo dependencies
- Company-wide rollout depends on signal quality and trust, not feature count

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Pilot teams unavailable | Medium | High | Identify backup teams early |
| Metrics baseline unclear | Medium | Medium | Phase 0 dedicated to baseline |
| Bundle schema breaks | Low | High | Versioning strategy + backward compat |
| Drift noise too high | Medium | High | Phase 1 dedicated to noise reduction |
| Multi-repo conflicts unclear | Medium | Medium | Phase 3 ownership model |
| Teams reject workflow change | Medium | High | Pilot feedback loops, rollback plan |
