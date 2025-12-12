# Real-World Examples for New Features

This directory contains realistic, production-ready examples demonstrating the new features:

## Implied Relationships

### `real_world_implied_relationships.sruja`

**Scenario:** Microservices E-commerce Platform

**What it demonstrates:**

- Complex microservices architecture with 8+ services
- Multiple data stores and message queues
- External service integrations
- How implied relationships reduce boilerplate in large architectures

**Key Benefits:**

- **Before:** Would need ~50+ explicit relationships
- **After:** Only ~30 explicit relationships needed (40% reduction)
- Automatically infers parent relationships (e.g., `Customer -> ECommerce` from `Customer -> ECommerce.WebApp`)

**Real-world use case:**
Perfect for documenting microservices architectures where you have many services and want to focus on specific interactions without repeating parent relationships.

### `real_world_microservices.sruja`

**Scenario:** E-commerce Microservices Platform

**What it demonstrates:**

- API Gateway pattern with multiple backend services
- Service-to-service communication
- Event-driven architecture
- How implied relationships simplify service mesh documentation

**Key Benefits:**

- Reduces relationship definitions by ~35%
- Makes service interactions clearer
- Automatically shows system-level relationships

## Views Block Customization

### `real_world_views_customization.sruja`

**Scenario:** SaaS Analytics Platform

**What it demonstrates:**

- **Multiple stakeholder views:**
  - Developer View: API and processing services
  - Product View: User-facing components
  - Data Flow View: Data pipeline and storage
  - Executive Overview: High-level system context

**Key Features:**

- Custom filtering for different audiences
- Tag-based styling (Database, Queue, external services)
- Visual differentiation by element type
- Multiple views in one architecture file

**Real-world use case:**
Perfect for large platforms where different teams need different views:

- **Developers:** Focus on APIs and services
- **Product Managers:** Focus on user experience
- **Data Engineers:** Focus on data flow
- **Executives:** High-level overview

## Combined Example

### `real_world_microservices.sruja`

**Scenario:** E-commerce Microservices (Combined Features)

**What it demonstrates:**

- Implied relationships for service interactions
- Custom views for different teams:
  - API Architecture (for API team)
  - Data Architecture (for data team)
  - External Dependencies (for operations team)
- Tag-based styling for visual clarity

## Usage

### Export Individual Examples

```bash
# Export as JSON (currently supported format)
sruja export json examples/real_world_implied_relationships.sruja

# Export with extended views
sruja export json --extended examples/real_world_views_customization.sruja

# Export to stdout
sruja export json examples/real_world_microservices.sruja > output.json
```

Note: Markdown, SVG, and HTML exports are currently disabled. Use the TypeScript exporters in frontend apps (playground, website) for these formats.

## Key Takeaways

### Implied Relationships

- **Best for:** Complex architectures with many nested elements
- **Benefit:** 30-40% reduction in relationship definitions
- **Use when:** You want to focus on specific interactions without boilerplate

### Views Block

- **Best for:** Large platforms with multiple stakeholders
- **Benefit:** One architecture, multiple views for different audiences
- **Use when:** Different teams need different levels of detail

### Combined

- **Best for:** Enterprise architectures
- **Benefit:** Simplified relationships + Customized views = Better documentation
- **Use when:** You need both simplicity and flexibility

## Comparison

| Feature                   | Simple Example             | Real-World Example             |
| ------------------------- | -------------------------- | ------------------------------ |
| **Implied Relationships** | 3 relationships, 1 implied | 30+ relationships, 10+ implied |
| **Views Block**           | 1 custom view              | 4 stakeholder-specific views   |
| **Complexity**            | Beginner-friendly          | Production-ready               |
| **Use Case**              | Learning                   | Documentation                  |

## Next Steps

1. **Try the examples:** Export them and see the results
2. **Adapt to your needs:** Use as templates for your architecture
3. **Combine features:** Use both implied relationships and views together
4. **Customize views:** Create views specific to your team's needs
