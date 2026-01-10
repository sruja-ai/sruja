# Sruja Designer: Architectural Assessment

Is the Designer app too complex? Does it need splitting? This assessment evaluates the current "Role-Based Monolith" strategy.

## 1. Current State: "Coherent Complexity"

The `designer` app is certainly large (350+ files), but its complexity is **logical and grouped**, rather than monolithic and tangled.

### Strengths of the Current Approach:

- **The Single Source of Truth**: All stakeholders (CTO, SRE, Architect) are looking at the same underlying state (`ArchitectureStore`). This ensures that when an Architect renames a service, the CTO’s "Technical Debt" report updates instantly.
- **Contextual Fluidity**: One of Sruja's unique values is the ability to switch "lenses" in one click. Splitting into separate apps (e.g., `cto.sruja.ai`) would introduce **role silos** and significant sync overhead.
- **Shared Infrastructure**: 90% of the app (Canvas, DSL Parser, WASM Engine, Theming) is shared across all roles. Splitting would require complex shared-library management or duplicate code.

## 2. Assessment: To Split or Not?

### Why it makes sense as ONE App:

1.  **Shared State**: The dependency on the WASM-powered Go engine is heavy. Loading this in 6 different apps would significantly increase browser memory usage if a user keeps multiple tabs open.
2.  **Shared UI Patterns**: The `Wizard`, `CodePanel`, and `SrujaCanvas` are the scaffolding. The role-specific views (e.g., `HealthScore`, `RiskAssessment`) are relatively thin layers (8-10 files each) on top of this scaffolding.
3.  **Discovery**: An Architect might not know they have a "Reliability Gap" until they toggle the SRE view. This discovery is lost in a split-app world.

### When a Split _Does_ Make Sense:

- **The "Consumer" View**: If you want a lightweight, read-only "Viewer" for documentation (with no DSL editor, no WASM logic, only SVG rendering), that should be a separate package/app for better performance.
- **The "Guided" Experience**: If the `BuilderWizard` grows into a massive AI-driven interview system, it could be split into a separate route or micro-frontend to improve initial load times for power users who just want the editor.

## 3. Final Recommendation

**Verdict: Do not split by role; refactor by package.**

The app is NOT too complex; it is **rich**. To keep it manageable as it grows:

1.  **Move more to `@sruja/ui`**: Keep the `designer` app focused only on _orchestrating_ views.
2.  **Lazy Load Roles**: Use React `Suspense` and dynamic imports for the role directories (`src/components/CTO`, `src/components/SRE`). This ensures that a Product Manager doesn't download the SRE logic unless they switch to that role.
3.  **Route-based Splitting**: Keep it in one Repo/Project, but use distinct routes for the `Wizard` vs. the `Editor` to isolate large dependencies.

### Closing Thought

Sruja’s power is it being a **collaborative canvas**. Splitting it is effectively splitting the conversation between and Architect and their SRE. Keep the conversation in one room.
