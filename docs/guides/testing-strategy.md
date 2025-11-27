# Testing Strategy

Comprehensive testing approach for the Architecture DSL Platform.

## Testing Philosophy

With a **DSL**, **parser**, **LSP**, and **bidirectional visual editor**, testing is mission-critical. The platform has three categories of tests:

1. **Unit Tests** → Parser, AST builder, serializer, model engine, LSP features
2. **Component Tests** → React Flow nodes, Monaco editor wrappers
3. **E2E Tests** → Full text ⇄ model ⇄ diagram workflow

## MVP Critical Tests (Required)

| Area             | Why Needed                     | Priority |
| ---------------- | ------------------------------ | -------- |
| DSL parser       | Correctness & safety           | 🔴 Critical |
| AST builder      | Stable structure               | 🔴 Critical |
| Model engine     | Diagram correctness            | 🔴 Critical |
| Validation engine | Semantic correctness          | 🔴 Critical |
| LSP diagnostics  | UX correctness                 | 🔴 Critical |
| LSP autocomplete  | Core usability                 | 🔴 Critical |
| Serializer       | Bidirectional sync correctness | 🔴 Critical |
| Round-trip        | Ensures canonical DSL          | 🔴 Critical |

## MVP Optional Tests (Nice to Have)

- E2E tests (1-2 simple scenarios)
- LSP hover tests
- Component tests (React Flow node rendering)

## Post-MVP Tests (Skip for Now)

- Full UI component coverage
- Multi-user sync tests
- Accessibility tests
- Heavy ADR/journey tests
- VS Code extension tests

## Testing Tools

- **Vitest 2.x**: Unit & integration tests (fast, Bun-compatible)
- **Playwright**: E2E tests (Next.js integration)
- **@testing-library/react**: Component tests
- **Snapshot testing**: For DSL parsing results

## Test Structure by Phase

### Phase 1 (DSL Foundation)
- Grammar tests (Ohm)
- AST builder tests
- Model validation tests (Zod)
- Round-trip tests (parse → model → serialize → parse)
- **Validation Engine tests**:
  - Test validation API (`validateArchitecture`)
  - Test semantic rules (unknown reference, duplicate ID, etc.)
  - Test layer rules
  - Test best-practice rules
  - Test plugin system
- Error handling tests

### Phase 2 (LSP Server)
- Diagnostics generation tests
- Position mapping tests (offset → line/column)
- Completion provider tests
- Hover provider tests
- Model syncing tests
- Error handling tests

### Phase 3 (Editor MVP)
- Component rendering tests (optional)
- Sync engine tests
- Undo/redo tests

### Phase 4 (Backend)
- API endpoint tests
- WebSocket sync tests
- Git storage tests
- Integration tests

### E2E Tests (1-2 critical scenarios)
- Simple DSL loads and displays diagram
- Drag node → DSL updates
- LSP diagnostics appear in Monaco

## Enterprise-Scale Example Suites

Use the following end-to-end models to stress-test multi-module composition, cross-context linking, domain boundaries, bounded contexts, team ownership, context maps, cross-module references, shared modules, external dependencies, anti-corruption layers, plugin rules, and import graphs.

### 1) E-Commerce Platform (Amazon-like)

- Domains: Identity, Ordering, Payments, Inventory, Logistics
- Contexts: identity.auth, ordering.checkout, ordering.orderManagement, payments.core, inventory.stock, logistics.fulfillment
- Modules: auth-service, storefront-web, checkout-web, order-service, payment-service, stock-service, warehouse-service
- Relationships:
```
storefront.web -> auth.api
storefront.web -> checkout.api
checkout.api -> payments.api
checkout.api -> order.api
order.api -> stock.api
order.api -> fulfillment.api
```
- Context Map:
```
identity.auth -> ordering.checkout: "customer-supplier"
ordering.orderManagement -> inventory.stock: "conformist"
ordering.checkout -> payments.core: "conformist"
inventory.stock -> logistics.fulfillment: "customer-supplier"
```

### 2) Ride-Sharing / Mobility App (Uber-like)

- Domains: Identity, Mobility Matching, Pricing, Payments, Location/Maps
- Contexts: identity.auth, mobility.dispatch, mobility.rider, mobility.driver, pricing.engine, payments.billing, location.geoservice
- Modules: auth-service, rider-app, driver-app, dispatch-service, pricing-engine, billing-service, map-engine
- Relationships:
```
rider.app -> auth.api
rider.app -> dispatch.api
dispatch.api -> driver.app
dispatch.api -> pricing.api
dispatch.api -> geoservice.api
billing.api -> payments.gateway
```

### 3) Online Banking / Fintech (Revolut-like)

- Domains: Identity, Accounts, Cards, Payments, Compliance, Risk
- Contexts: identity.auth, accounts.core, cards.issuer, payments.transfer, compliance.aml, risk.fraud
- Modules: identity-service, accounts-service, cards-service, payments-service, aml-service, fraud-engine, web-banking-app
- Relationships:
```
web.app -> identity.api
web.app -> accounts.api
payments.api -> fraud.api
payments.api -> aml.api
cards.api -> accounts.api
```

### 4) Social Network (Twitter/X-like)

- Domains: Identity, Profiles, Social Graph, Timeline, Media, Notifications
- Contexts: identity.auth, profiles.core, social.relationships, timeline.feed, media.upload, notifications.dispatch
- Modules: auth-service, user-profile-service, follow-service, feed-service, media-service, notification-service, web-client, mobile-app
- Relationships:
```
web.ui -> auth.api
web.ui -> profile.api
web.ui -> feed.api
feed.api -> social.api
feed.api -> media.api
notification.api -> feed.api
```

### 5) Online Learning Platform (Coursera-like)

- Domains: Identity, Course Management, Content Delivery, Payments, Engagement
- Contexts: identity.auth, courses.catalog, courses.learning, delivery.media, payments.billing, engagement.quiz
- Modules: auth-service, catalog-service, learning-engine, media-delivery-service, quiz-service, web-frontend, instructor-dashboard
- Relationships:
```
web.ui -> auth.api
web.ui -> catalog.api
web.ui -> learning.api
learning.api -> media.api
learning.api -> quiz.api
quiz.api -> learning.api
billing.api -> catalog.api
```

### 6) Project Management SaaS (Jira-like)

- Domains: Identity, Projects, Issues, Sprints, Notifications, Analytics
- Contexts: identity.auth, projects.core, issues.tracking, sprints.planning, notifications.dispatch, analytics.metrics
- Modules: auth-service, project-service, issue-service, sprint-service, notification-service, analytics-service, react-web
- Relationships:
```
react.ui -> auth.api
react.ui -> issues.api
issues.api -> projects.api
sprints.api -> issues.api
analytics.api -> issues.api
notifications.api -> issues.api
```

### 7) HealthTech / EMR (Epic-like)

- Domains: Identity, Patients, Appointments, Records, Billing, Lab Results
- Contexts: identity.auth, patients.core, appointments.scheduling, records.emr, billing.claims, labs.results
- Modules: auth, patient-service, appointment-service, emr-service, billing-service, lab-service, doctor-web, patient-portal
- Relationships:
```
doctor.web -> identity.api
doctor.web -> patients.api
patients.api -> emr.api
appointments.api -> patients.api
emr.api -> labs.api
billing.api -> emr.api
```

### 8) Travel Booking (Booking.com-like)

- Domains: Identity, Search, Booking, Payments, Inventory, Reviews
- Contexts: identity.auth, search.engine, booking.core, payments.core, inventory.hotels, reviews.feedback
- Modules: search-service, booking-service, review-service, hotel-inventory-service, payment-service, auth-service, web-app
- Relationships:
```
web.app -> search.api
search.api -> inventory.api
booking.api -> payments.api
booking.api -> inventory.api
reviews.api -> booking.api
```

### 9) CI/CD Platform (GitHub Actions-like)

- Domains: Identity, Repositories, Workflows, Runners, Billing, Artifacts
- Contexts: identity.accounts, repos.core, workflows.pipeline, runners.agent, billing.usage, artifacts.storage
- Modules: repo-service, workflow-service, runner-service, artifact-service, billing-service, auth-service, vscode-extension
- Relationships:
```
vscode -> auth.api
vscode -> repos.api
workflow.api -> repos.api
workflow.api -> runners.api
runner.api -> artifacts.api
billing.api -> workflow.api
```

### 10) Messaging / Chat Platform (Slack-like)

- Domains: Identity, Messaging, Channels, Notifications, Media, Search
- Contexts: identity.auth, messaging.core, channels.groups, notifications.alerts, media.store, search.index
- Modules: messaging-service, channel-service, media-service, search-service, notifier, auth, web-client, mobile-app
- Relationships:
```
web.ui -> auth.api
web.ui -> messaging.api
messaging.api -> channels.api
messaging.api -> notifications.api
messaging.api -> search.api
media.api -> messaging.api
```

### Coverage Table

| Example | Tests | Notes |
| --- | --- | --- |
| E-commerce | cross-context chaining | end-to-end lifecycle |
| Ride-sharing | real-time + multi-app | fan-out relationships |
| Banking | regulated boundaries | AML + fraud rules |
| Social network | complex graph | cross-domain traffic |
| Learning platform | bi-directional contexts | quizzes + lessons |
| PM SaaS | inter-module | validation-rich |
| HealthTech | privacy rules | HIPAA-like checks |
| Booking | search → inventory → booking | multi-step flows |
| CI/CD | runners + billing | upstream/downstream |
| Chat | messaging flow | indexing + media |

## Example Test Structure

```typescript
describe("DSL Parsing", () => {
  it("parses simple service definition", () => {
    const model = parse(`api: service "API"`);
    expect(model.nodes[0]).toMatchSnapshot();
  });

  it("detects unknown type", () => {
    expect(() => parse(`x: weirdType`)).toThrowError();
  });
});
```

## LSP Testing Example

```typescript
it("reports unknown identifier", async () => {
  const diagnostics = await lsp.getDiagnostics(`
    api -> missingService
  `);
  expect(diagnostics.length).toBe(1);
  expect(diagnostics[0].message).toContain("missingService not defined");
});
```

## MCP Integration Scenarios

### 1) Explain architecture (read-only)
- MCP: `read_model()`
- AI: Summarize domains → contexts → modules → relationships
- Expected: Overview with major flows

### 2) List services in Payments domain
- MCP: `query({ filter: { domain: "payments" } })`
- Expected: `[ { module: "paymentService", context: "payments.corepay" }, { module: "billingService", context: "payments.billing" } ]`

### 3) Who depends on Booking Service
- MCP: `query({ relationship: { target: "bookingService" } })`
- Expected: `[ { from: "reviewService", type: "depends_on" }, { from: "paymentService", type: "uses" } ]`

### 4) Detect cyclic dependencies
- MCP: `read_model()` → `validate({ rule: "no-cyclic-dependencies" })`
- Expected error: `bookingService → reviewService → bookingService`

### 5) Cross-domain violations
- MCP: `validate({ rule: "no-cross-domain-without-ACL" })`
- Expected warning: `orders.orderService → payments.paymentService`

### 6) PII without encryption
- MCP: `validate({ rule: "data-at-rest-encryption" })`
- Expected: `[ { module: "patientService.db", issue: "No encryption: true" } ]`

### 7) Add FraudService and wire into payments
- MCP: `read_model()` → `update_model({ add: { module: { id: "fraudService", context: "risk.fraud", containers: [{ id: "api", kind: "Service" }] } }, link: { from: "paymentsService.api", to: "fraudService.api", label: "fraud-check" } })` → `validate()`
- Expected: Created service + link; no validation errors

### 8) Split checkout-service
- MCP: `read_model()` → `update_model({ transform: { split: { module: "checkoutService", into: [ { id: "checkoutApi", containers: [...existing] }, { id: "checkoutWorker", containers: [{ id: "worker", kind: "Worker" }] } ] } } })` → `validate()`
- Expected: Two modules, rewired dependencies

### 9) Map Create Order journey
- MCP: `query({ journey: "CreateOrder" })`
- Expected: `[ "storefrontWeb", "authService", "checkoutService", "paymentService", "inventoryService", "orderService", "logisticsService" ]`

### 10) Simulate Payments down
- MCP: `query({ simulate_failure: "paymentService" })`
- Expected: Affected flows: checkoutService, orderService, storefrontWeb

### 11) Create ADR for event sourcing
- MCP: `read_model()` → `update_model({ add: { adr: { id: "ADR-004", title: "Introduce Event Sourcing in Order Context", status: "Proposed", context: "...", decision: "...", consequences: ["Complexity ↑", "Auditability ↑"] } } })`
- Expected: `ADR-004 created under /adrs`

### 12) Suggest decoupling improvements
- MCP: `read_model()` → query SCCs → propose refactors → optional `update_model()`
- Expected: `Introduce ACL between checkoutService and paymentService`; move shared DTOs into `shared.order-types`

## Model Engine Testing Example

```typescript
it("prevents adding component without container", () => {
  expect(() => model.addComponent("c1")).toThrowError();
});
```

---

[← Back to Documentation Index](../README.md)

