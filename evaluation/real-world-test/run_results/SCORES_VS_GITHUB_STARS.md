# Are Sruja Scores Justified by GitHub Stars?

**Short answer:** **No — and by design.** Sruja’s health score does **not** use GitHub stars (or any popularity metric). It is based only on **structural violations** in the scanned codebase. So scores are not “justified as per GitHub stars”; they are justified (or not) by **how well the code structure matches the scoring rules**.

---

## 1. How the score is computed

**Source:** `crates/sruja-diff/src/health.rs`

- **Input:** List of violations from the scan (cycles, orphans, god modules, layer violations).
- **Formula:** Start at 100, subtract **capped** penalties:
  - Cycles: −2 each, **max −10**
  - Layer violations: −3 each, **max −10**
  - Orphans: −1 each, **max −10**
  - God modules (>10 deps): −1 per 50, **max −10**
  - Other: **max −10**
- **Floor:** `score.max(MIN_SCORE)` with **MIN_SCORE = 50** (“successful projects deserve respect”).
- **Density-based path:** A `calculate_health_score_with_density` exists (issues per 1000 modules) so that “a 54k star project with 15K modules getting 40/100 is embarrassing and wrong,” but it is **not currently used** in the CLI; the code path is `calculate_health_score_from_violations` (absolute counts with caps).

So: **GitHub stars are never read or used.** The score is purely “100 minus structural penalties, floored at 50.”

---

## 2. Comparison: GitHub stars vs Sruja score (observed)

| Repo        | GitHub stars (approx) | Sruja health (run) | Why the score? |
|------------|------------------------|--------------------|-----------------|
| **express** | ~69k                   | **93**             | 85 modules, 7 orphans (cap −7), no cycles → 100−7=93. |
| **gitea**   | ~54k                   | **78**             | 15k modules, 1 cycle (−2), 27 orphans (cap −10), 892 god (cap −10) → 100−22=78 (capped). |
| **etcd / caddy** (docs) | Very high (etcd ~47k, caddy ~35k) | **0/100** (older runs) | Many orphans (doc/tools packages) drove score down; doc says this was “overly harsh.” With current MIN_SCORE=50 and caps, they would now get **≥50**. |

So:

- **High stars, high score:** Express (69k stars, 93) — few violations.
- **High stars, lower score:** Gitea (54k stars, 78) — more violations (god modules, orphans, one cycle).
- **High stars, historically very low score:** etcd/caddy — many violations (orphans), no correlation with stars.

There is **no built-in correlation** between stars and score. Any alignment is incidental (e.g. popular repos that also have clean structure).

---

## 3. Are the scores “justified as per GitHub stars”?

- **If “justified” means “does the score use or match stars?”**  
  **No.** The score is not derived from stars and is not meant to reflect popularity.

- **If “justified” means “is it fair that a 50k-star repo gets 78?”**  
  **Yes, given the design.** The score is intended to reflect **architecture/structure** (cycles, orphans, god modules, layers). Gitea’s 78 reflects real structural findings (e.g. 892 god modules, orphans); it is not a judgment on “how good the project is” in a popularity sense.

- **If “justified” means “should high-star repos get high scores?”**  
  Only if their **structure** is clean. The codebase explicitly aims to avoid unfairly punishing large, successful projects (MIN_SCORE 50, density philosophy in comments), but the **current** implementation does not use density for scoring in the CLI, so very violation-heavy repos could still land near the floor (50).

**Bottom line:** Scores are **justified by the structural rules**, not by GitHub stars. They are “justified as per stars” only in the weak sense that the design tries to avoid absurdly low scores for successful projects (floor 50); they are **not** calibrated or normalized by popularity.

---

## 4. Recommendations

1. **Interpretation:** Treat the health score as an **indicative structural signal**, not as “this repo is good/bad by popularity.” Use **findings + drift vs architecture** for decisions (see TEST_ON_REAL_PROJECTS.md).
2. **If you want scores to “respect” popularity:** The code already has a density-based scoring path and a MIN_SCORE; wiring density-based scoring for large repos (e.g. when `total_modules >= 100`) would better match the comment “well-maintained projects score 70+.”
3. **If you want scores to correlate with stars:** That would require either (a) using star count (or similar) as an explicit input to the score (not currently done and would change the meaning of “architecture health”), or (b) accepting that correlation is only indirect (e.g. popular repos often have cleaner structure).

---

*Data: Express/Gitea stars from GitHub API (2026-02-27). Sruja scores from quickstart/drift runs in this evaluation.*
