# E2E diagram smoke (CI)

Minimal Sruja block used by deploy E2E to verify WASM + Mermaid in the built book.
Not linked from the public nav; referenced only by Playwright.

```sruja
Shop = system "Shop" {
  description "Example system for CI diagram smoke"

  Web = container "Web" {
    technology "React"
    description "Storefront"
  }

  API = container "API" {
    technology "Rust"
    description "Backend API"
  }
}

Buyer = person "Buyer" {
  description "Customer"
}

Buyer -> Shop.Web "browses"
Shop.Web -> Shop.API "calls"
```
