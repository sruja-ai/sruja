use crate::commands::CliError;

const QUESTION_BANK: &str = r#"# Sruja discovery question bank

Ask the user 2–5 of these (adapt to context). Use answers to set scope, subpath, names, and externals.

## Context / shape
- Is this a single service, a monolith with modules, or several microservices?
- Should we capture one area first or the whole repo?

## Large repo
- The repo is big. Should we focus on a specific area (e.g. services/auth, apps/web) or the whole codebase? I can capture by subpath and we can stitch later.
- Which directory or service should we start with?

## Scope
- Do you want a minimal sketch (entry points + main deps), standard (10–30 components), or a deeper model (internal layers, error paths)?

## Boundaries
- What are your main bounded contexts or team-owned areas?
- Any external systems (payments, auth, notifications) that must appear in the diagram?

## Entry points and flows
- What's the main user-facing entry (web app, public API, CLI)?
- Any key flows (e.g. checkout, auth) I should make explicit?

## Refinement (after first draft)
- Does this match how you think about the system? Any services or boundaries missing?
- Prefer different names for systems or containers?

---
Use with: npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
Then in Cursor: run the agent and ask it to discover architecture; it will use this question bank.
"#;

pub fn discover_questions() -> Result<(), CliError> {
    println!("{}", QUESTION_BANK);
    Ok(())
}
