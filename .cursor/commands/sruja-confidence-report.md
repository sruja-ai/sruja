# Sruja Confidence Report

Generate a post-AI-edit confidence report showing what changed, what evidence was checked, what risks remain, and what to inspect at 3AM.

## When to use

After an AI coding assistant (Cursor, Copilot, Claude, etc.) makes changes to your codebase. Run this before reviewing the diff to get a structured overview of verification status.

## Workflow

```
1. AI edits code (in your editor)
2. sruja confidence -r . -f md          ← this command
3. Review the report
4. Address blockers if any
5. Merge when confident
```

## Profiles

- `review` (default) — review + intent check + drift check
- `coding` — lint repo.sruja + build/test + drift check
- `bugfix` — focus on file + build/test + intent check (requires `--file`)
- `arch` — lint + drift + intent + review

## Commands

```bash
# Default: review profile, markdown output
sruja confidence -r .

# JSON output for tooling
sruja confidence -r . -f json

# Bugfix profile with focus file
sruja confidence --profile bugfix --file src/auth.rs -r .

# With evidence pack
sruja confidence --profile coding --evidence-pack -r .
```

## Report sections

- **Verdict** — confidence level (high/medium/low) and summary
- **What Changed** — list of changed files from git diff
- **Intent Alignment** — intent check results
- **Architecture Alignment** — drift check results
- **Evidence Checked** — verification steps that ran
- **Human Review Queue** — blockers and watch items
- **3AM Notes** — first places to check and follow-up commands

## Confidence levels

- **High** — all verification passed, no blockers, intent is clear
- **Medium** — verification passed but some signals are missing or unclear
- **Low** — verification failed, drift detected, or intent errors found

## Advisory behavior

The command is advisory by default: it exits successfully even if the report contains blockers. It only exits non-zero for fatal execution/input errors (e.g., repo not found).

## Integration with existing workflow

```bash
# Before AI edits (optional)
sruja focus --file src/auth.rs -r .

# AI edits happen here...

# After AI edits
sruja confidence -r . -f md

# If blockers found, investigate
sruja drift -r . -f json
sruja intent check -r . -f json
```
