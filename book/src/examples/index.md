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

## Examples

| Example                                                | Description                                         | Views                                                      |
| ------------------------------------------------------ | --------------------------------------------------- | ---------------------------------------------------------- |
| [Advanced Views](./examples/advanced-views.md)         | Multiple custom views with include/exclude patterns | API Focus, Customer Experience, System Context, Data Layer |
| [Basic System](./examples/basic-system.md)             | Simple person and system with relations             | System Context                                             |
| [Containers](./examples/containers.md)                 | System with nested containers                       | System, Containers                                         |
| [Databases](./examples/databases.md)                   | Systems with database components                    | System, Database View                                      |
| [Scenarios](./examples/scenarios.md)                   | Scenario flows with sequence diagrams               | All Scenarios                                              |
| [Feedback Loops](./examples/feedback-loops.md)         | Systems thinking feedback loops                     | All Loops                                                  |
| [Causal Loops](./examples/causal-loops.md)             | Causal loop diagrams                                | All Loops                                                  |
| [Metadata](./examples/metadata.md)                     | Elements with metadata and tags                     | Full Model                                                 |
| [Datastores & Queues](./examples/datastores-queues.md) | Complex systems with datastores and queues          | Full Architecture                                          |
| [Deployment](./examples/deployment.md)                 | Deployment configurations                           | Infrastructure                                             |
| [Governance](./examples/governance.md)                 | Policies and constraints                            | Governance View                                            |
| [Microservices](./examples/microservices.md)           | Microservice architecture                           | Service Views                                              |
| [E-commerce](./examples/ecommerce.md)                  | Full e-commerce platform                            | Context, Containers, Components                            |

## How to Add Your Own Examples

1. Create a `.sruja` file in the `examples/` directory
2. Create a corresponding `.md` file in `book/src/examples/`
3. Use the sruja-export directive:

```markdown
# Your Example Name

Example description here.

<!-- sruja:export ../../examples/your-example.sruja --all-views -->
```

4. Rebuild the book: `mdbook build`
