---
title: "How Sruja Works"
weight: 3
---

# How Sruja Works

Sruja is built to be a **tool for the AI SDLC process**: architecture in code that fits into your lifecycle—IDE, CI/CD, and documentation. We are not a diagramming product; we provide parse, validate, export, and optional preview.

## The Sruja Platform

The platform consists of several key components working together:

1.  **Parser & engine**: Rust crates for parsing, validation, and export (sruja-language, sruja-engine, sruja-export).
2.  **CLI**: Command-line interface for local development and CI/CD (sruja-cli).
3.  **WASM**: Rust core compiled to WebAssembly for the docs book and VS Code (sruja-wasm).
4.  **LSP**: Language server for VS Code (sruja-lsp).
5.  **Docs**: This site—built with mdBook from the `book/` directory.

## Architecture Diagram

Explore the Sruja architecture itself using the interactive viewer below. This diagram is defined in Sruja DSL!

```sruja
import { * } from 'sruja.ai/stdlib'


RootSystem = system "The Sruja Platform" {
  tags ["root"]
}

User = person "Architect/Developer" {
	description "Uses Sruja to design and document systems"
}

Sruja = system "Sruja Platform" {
	description "Tools for defining, visualizing, and analyzing software architecture"

	CLI = container "Sruja CLI" {
		technology "Rust"
		description "Command-line interface (crates/sruja-cli)"
	}

	Engine = container "Core Engine" {
		technology "Rust"
		description "Validation and export (crates/sruja-engine, sruja-export)"

		Validation = component "Validation Engine" {
			technology "Rust"
			description "Validates AST against rules (crates/sruja-core/src/engine/rules)"
		}

		Scorer = component "Scoring Engine" {
			technology "Rust"
			description "Calculates architecture health score (crates/sruja-core/src/engine/scorer)"
		}

		Policy = component "Policy Engine" {
			technology "Rust"
			description "Enforces custom policies (future: OPA/Rego)"
		}

		Scorer -> Validation "uses results from"
		Validation -> Policy "checks against"
	}

	Language = container "Language Service" {
		technology "Rust"
		description "Parser and AST (crates/sruja-language); LSP (crates/sruja-lsp)"
	}

	WASM = container "WASM Module" {
		technology "Rust/WASM"
		description "WebAssembly build (crates/sruja-wasm)"
	}

	VSCode = container "VS Code Extension" {
		technology "TypeScript"
		description "Editor extension (extension/)"
	}

	Book = container "Documentation" {
		technology "mdBook"
		description "This site (book/)"
	}

	// Internal Dependencies
	CLI -> Language "parses DSL using"
	CLI -> Engine "validates using"
	CLI -> WASM "builds"

	WASM -> Language "embeds"
	WASM -> Engine "embeds"

	VSCode -> Language "uses LSP"
	VSCode -> WASM "uses for LSP and preview"

	Book -> WASM "uses for diagram blocks"
}

User -> Sruja.CLI "runs commands"
User -> Sruja.VSCode "writes DSL"
User -> Sruja.Book "reads docs"

BrowserSystem = system "Web Browser" {
	description "User's web browser environment"
  tags ["external"]
	LocalStore = database "Local Storage"
}

// ADRs
ADR001 = adr "Use WASM for Client-Side Execution" {
	status "Accepted"
	context "We need to run validation and parsing in the browser and VS Code without a backend server."
	decision "Compile the Rust core engine to WebAssembly."
	consequences "Ensures consistent logic across all platforms but increases build complexity."
}

// Deployment
deployment Production "Production Environment" {
  node GitHubPages "GitHub Pages" {
    containerInstance RootSystem
  }
}

GitHubSystem = system "GitHub Platform" {
  description "Source control, CI/CD, and hosting"
  Actions = container "GitHub Actions" {
    technology "YAML/Node"
    description "CI/CD workflows"
  }
  Pages = container "GitHub Pages" {
    technology "Static Hosting"
    description "Hosts documentation site"
  }
  Releases = container "GitHub Releases" {
    technology "File Hosting"
    description "Hosts CLI binaries"
  }
  Actions -> Pages "deploys to"
  Actions -> Releases "publishes to"
}


User -> GitHubSystem "pushes code to"


// Component Stories
CLIStory = story "Using the CLI" {
  User -> Sruja.CLI "runs validate"
  Sruja.CLI -> Sruja.Language "parses DSL"
  Sruja.CLI -> Sruja.Engine "validates"
  Sruja.CLI -> User "reports diagnostics"
}

VSCodeStory = story "Using VS Code" {
  User -> Sruja.VSCode "edits .sruja file"
  Sruja.VSCode -> Sruja.WASM "LSP and preview"
  Sruja.WASM -> Sruja.VSCode "diagnostics and diagram"
  Sruja.VSCode -> User "shows errors and preview"
}

CIDev = scenario "Continuous Integration (Dev)" {
  User -> GitHubSystem "pushes to main"
  GitHubSystem -> GitHubSystem.Actions "triggers CI"
  GitHubSystem.Actions -> Sruja "builds & tests"
  GitHubSystem.Actions -> GitHubSystem.Pages "deploys dev site"
}

ReleaseProd = scenario "Production Release" {
  User -> GitHubSystem "merges PR to prod"
  GitHubSystem -> GitHubSystem.Actions "triggers release"
  GitHubSystem.Actions -> GitHubSystem.Pages "deploys prod site"
  GitHubSystem.Actions -> Sruja.VSCode "publishes extension"
  GitHubSystem.Actions -> GitHubSystem.Releases "publishes CLI binaries"
}

view index {
  title "Complete System View"
  include *
}
```

## Key Components

### Core Engine (Rust)

The [`sruja-language`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-language) and [`sruja-engine`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-engine) crates form the foundation. They define the DSL grammar, parse input files into an AST (Abstract Syntax Tree), and run validation rules (like cycle detection and layer enforcement).

### WebAssembly (WASM)

The Rust core is compiled to WebAssembly ([`sruja-wasm`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-wasm)). The same parsing and validation logic runs in:

- **VS Code Extension**: For local preview without needing a CLI binary.
- **Documentation site**: For "Show diagram" in code blocks (like the one above).

### CLI & CI/CD

The `sruja` CLI ([`sruja-cli`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-cli)) is a static binary that wraps the core engine. It supports:

- **Local development**: `sruja fmt`, `sruja lint`, `sruja export`.
- **CI/CD**: Validate and export architecture in pipelines.
- **Export**: `sruja export json`, `sruja export mermaid`, `sruja export markdown`, `sruja export context`, `sruja export dsl`.
