---
title: "Examples Gallery"
weight: 47
summary: "Browse and interact with Sruja architecture examples."
tags: ["examples"]
---

# Examples Gallery

This page contains Sruja architecture examples that can be viewed and exported directly in mdBook.

## Usage

Click any example below to see the rendered architecture views with Mermaid diagrams.

Examples are rendered on-demand using the `<!-- sruja:export -->` directive in markdown.

Example source `.sruja` files live under `book/valid-examples/` (canonical, lint-clean).

## Examples

| Example                                                | Description                                         | Views                                                      |
| ------------------------------------------------------ | --------------------------------------------------- | ---------------------------------------------------------- |
| [Advanced Views](./advanced-views.md)                  | Multiple custom views with include/exclude patterns | API Focus, Customer Experience, System Context, Data Layer |
| [Basic System](./basic-system.md)                      | Smallest runnable example                           | System Context                                             |
| [Scenarios](./scenarios.md)                            | Scenario flows with steps                           | All Scenarios                                              |
| [Checkout Saga](./checkout-saga.md)                    | Saga-style orchestration example                    | System Context                                             |
| [Feedback Loops](./feedback-loops.md)                  | Reinforcing + balancing loops                       | All Loops                                                  |
| [Causal Loops](./causal-loops.md)                      | Polarity + delays                                   | All Loops                                                  |
| [Deployment](./deployment.md)                          | Deployment nodes and instances                       | Infrastructure                                             |
| [Governance](./governance.md)                          | ADRs + requirements + policies                      | Governance View                                            |
| [SLOs](./slo.md)                                       | Reliability targets + current state                 | Full Model                                                 |

## How to Add Your Own Examples

1. Create a `.sruja` file in `book/valid-examples/`
2. Create a corresponding `.md` file in `book/src/examples/`
3. Use the sruja-export directive:

```markdown
# Your Example Name

Example description here.

<!-- sruja:export valid-examples/your-example.sruja --all-views -->
```

4. Rebuild the book: `mdbook build`
