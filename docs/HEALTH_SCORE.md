# Is the health score meaningful?

Short answer: **yes, for a narrow definition of “health”** — it is a **structural** score based on a fixed set of violation types. It is consistent and interpretable, but it has deliberate limitations and is **not** a general “code quality” or “architecture quality” metric.

---

## What it is

The health score is a **0–100** value computed from the **drift report**: cycles, layer violations, god modules, and orphans (and a small “other” bucket). Formula (see `crates/sruja-diff/src/health.rs`):

- **Start at 100**, subtract penalties, **floor at 30**.
- **Cycles:** 2 points per circular dependency, **cap 15**.
- **Layer violations:** 1 point per violation, **cap 10**.
- **God modules:** `(count / 100)` points, **cap 10**.
- **Orphans:** `(count / 100)` points, **cap 5**.
- **Other** (unclassified violations by severity): **cap 8**.

So the score is **deterministic** and **reproducible** for a given scan. It is **not random**: the same repo yields the same score every time, and you can verify the number from violation counts (e.g. react-admin: 9 cycles → penalty 15, 186 god modules → penalty 1, total 16 → score 84). It is tuned so that:

- **Cycles and layer violations** have strong impact (they are treated as serious).
- **God modules and orphans** are damped (divided by 100, then capped), so large repos don’t collapse to 0 just because they have many modules; 90+ is intended to mean “few serious issues.”

Rough bands (from the code’s `HealthGrade`): 0–30 Critical, 31–50 Poor, 51–65 Fair, 66–80 Good, 81+ Excellent. The CLI may show different labels (e.g. “Good” at 99); the important part is that **higher = fewer/damped structural issues**.

---

## What it is good for

| Use | Why it helps |
|-----|-------------------|
| **Trend over time** | Same repo, same formula → if the score drops (e.g. in `drift-pr`), something structural got worse (e.g. new cycles or layer violations). |
| **Comparing refs** | Baseline vs head: “Health went from 85 to 78” is a clear signal to look at new violations. |
| **CI gate** | Fail when score falls below a threshold or when new violations appear; the score is stable enough to use as one input. |
| **Rough triage** | Low score (e.g. &lt; 60) → many cycles/layer violations; high score (e.g. 95+) → few of those, and god/orphan impact is capped. |

So for **structural drift** (cycles, layers, god modules, orphans), the score is **meaningful and useful** as a single number that reflects “how bad is the structural picture according to these rules.”

---

## What it is not

- **Not size-normalized in the default path.** The code has a density-based variant (`calculate_health_score_with_density`) that scores per 1000 modules, but the CLI uses the **violation-count** formula. So a tiny repo with 2 cycles loses the same 4 points as a huge repo with 2 cycles. The god/orphan caps (divide by 100) partly compensate in large repos.
- **Not a general “architecture quality” score.** It ignores: coupling strength, test coverage, documentation, deployment risk, naming, domain boundaries, etc. It is **structural only** (cycles, layers, god modules, orphans).
- **Not calibrated to your domain.** Thresholds (e.g. god module &gt; 10 deps) and caps are fixed. A 99 can mean “no cycles, no layer violations, and god/orphan counts damped by /100,” which may or may not match your idea of “excellent.”
- **Possible mismatch with labels.** The code’s grade bands (e.g. 66–80 = Good) and any prose (e.g. “40–60 = Critical”) may differ slightly; the **number** is the source of truth.

---

## Practical takeaway

- **Use the score for:** “Did we get worse between these two commits?” (e.g. `drift-pr`), and as one CI signal for structural drift.
- **Don’t use it as:** the only measure of “good architecture,” or as a size-normalized quality score, without reading the violation list.
- **Interpret it as:** “Structural health under Sruja’s rules: cycles and layer violations matter most; god modules and orphans are damped.” Within that scope, **yes — the health score is meaningful**.
