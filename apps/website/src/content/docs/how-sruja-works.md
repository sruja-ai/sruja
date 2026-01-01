---
title: "How Sruja Works"
weight: 3
---

# How Sruja Works

Sruja is built on a modern, modular architecture designed to bring **Architecture-as-Code** to every part of your development lifecycle—from your IDE to your CI/CD pipeline and documentation site.

## The Sruja Platform

The platform consists of several key components working together:

1.  **Core Engine**: The heart of Sruja, written in Go. It handles parsing, validation, and analysis.
2.  **CLI**: The command-line interface for local development and CI/CD integration.
3.  **WASM Module**: The core engine compiled to WebAssembly, enabling Sruja to run in the browser and VS Code.
4.  **Language Service**: Provides IDE features like syntax highlighting, auto-completion, and diagnostics.
5.  **Studio & Viewer**: Web-based tools for visualizing and interacting with your architecture.

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
		technology "Go"
		description "Command-line interface (cmd/sruja)"
	}

	Engine = container "Core Engine" {
		technology "Go"
		description "Core logic for validation, scoring, and analysis (pkg/engine)"

		Validation = component "Validation Engine" {
			technology "Go"
			description "Validates AST against rules (pkg/engine/rules)"
		}

		Scorer = component "Scoring Engine" {
			technology "Go"
			description "Calculates architecture health score (pkg/engine/scorer.go)"
		}

		Policy = component "Policy Engine" {
			technology "Go"
			description "Enforces custom policies (future: OPA/Rego)"
		}

		Scorer -> Validation "uses results from"
		Validation -> Policy "checks against"
	}

	Language = container "Language Service" {
		technology "Go"
		description "Parser, AST, and LSP implementation (pkg/language)"
	}

	WASM = container "WASM Module" {
		technology "Go/WASM"
		description "WebAssembly build of the core engine (cmd/wasm)"
	}

	VSCode = container "VS Code Extension" {
		technology "TypeScript"
		description "Editor extension (apps/vscode-extension)"
	}

	Designer = container "Sruja Designer" {
		technology "React/Vite"
		description "Interactive architecture design tool (apps/designer)"
	}

	Website = container "Documentation Site" {
		technology "Astro"
		description "Project documentation and guides (apps/website)"
	}

	// Internal Dependencies
	CLI -> Language "parses DSL using"
	CLI -> Engine "validates using"
	CLI -> WASM "builds"

	WASM -> Language "embeds"
	WASM -> Engine "embeds"

	VSCode -> Language "uses LSP"
	VSCode -> WASM "uses for LSP and preview"

	Designer -> WASM "uses for parsing/rendering"

	Website -> Designer "embeds"
}

User -> Sruja.CLI "runs commands"
User -> Sruja.VSCode "writes DSL"
User -> Sruja.Designer "designs architecture"
User -> Sruja.Website "reads docs"

BrowserSystem = system "Web Browser" {
	description "User's web browser environment"
  tags ["external"]
	LocalStore = database "Local Storage"
}

// ADRs
ADR001 = adr "Use WASM for Client-Side Execution" {
	status "Accepted"
	context "We need to run validation and parsing in the browser and VS Code without a backend server."
	decision "Compile the Go core engine to WebAssembly."
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
DesignerStory = story "Using Sruja Designer" {
  User -> Sruja.Website "visits designer"
  Sruja.Website -> Sruja.Designer "initializes"
  Sruja.Designer -> Sruja.WASM "loads engine"
  Sruja.Designer -> Sruja.WASM "parses DSL"
  Sruja.Designer -> User "renders interactive diagram"
}

DesignerExports = story "Exporting from Designer" {
  User -> Sruja.Designer "clicks export"
  Sruja.Designer -> Sruja.WASM "requests JSON"
  Sruja.WASM -> Sruja.Designer "returns JSON data"
  Sruja.Designer -> User "downloads file"
}

// Designer-specific Scenarios
DesignerShare = scenario "Share From Designer" {
  User -> Sruja.Designer "edits DSL code"
  Sruja.Designer -> BrowserSystem "updates URL with code"
  User -> Sruja.Designer "clicks Share button"
  Sruja.Designer -> BrowserSystem "copies shareable URL"
  User -> User "shares URL with team"
}

DesignerFormatPreview = scenario "Preview Multiple Formats" {
  User -> Sruja.Designer "loads architecture"
  User -> Sruja.Designer "switches to JSON preview"
  Sruja.Designer -> Sruja.WASM "parses DSL to JSON"
  Sruja.Designer -> User "displays JSON export"
}

DesignerURLState = scenario "URL State Management" {
  User -> Sruja.Designer "edits DSL code"
  Sruja.Designer -> BrowserSystem "debounces URL update"
  BrowserSystem -> BrowserSystem "updates URL hash with code"
  User -> BrowserSystem "refreshes page"
  BrowserSystem -> Sruja.Designer "loads code from URL"
  Sruja.Designer -> Sruja.WASM "parses DSL from URL"
  Sruja.Designer -> User "restores architecture state"
}

DesignerEditingStory = story "Visual Editing in Designer" {
  User -> Sruja.Designer "opens editor"
  Sruja.Designer -> Sruja.WASM "loads engine"
  User -> Sruja.Designer "types DSL code"
  Sruja.Designer -> Sruja.WASM "validates code"
  Sruja.WASM -> Sruja.Designer "returns diagnostics"
  Sruja.Designer -> User "shows errors/diagram"
}

DesignerAutosave = scenario "Autosave on Close" {
  Sruja.Designer -> BrowserSystem.LocalStore "save DSL snapshot"
  User -> Sruja.Designer "closes tab"
  User -> Sruja.Designer "reopens designer"
  Sruja.Designer -> BrowserSystem.LocalStore "load DSL snapshot"
  Sruja.Designer -> User "restores session"
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

### Core Engine (Go)

The [`pkg/engine`](https://github.com/sruja-ai/sruja/tree/main/pkg/engine) and [`pkg/language`](https://github.com/sruja-ai/sruja/tree/main/pkg/language) packages form the foundation. They define the DSL grammar, parse input files into an AST (Abstract Syntax Tree), and run validation rules (like cycle detection and layer enforcement).

### WebAssembly (WASM)

To ensure a consistent experience across all tools, we compile the Go engine to WebAssembly ([`cmd/wasm`](https://github.com/sruja-ai/sruja/tree/main/cmd/wasm)). This allows the **exact same parsing and validation logic** to run in:

- **Sruja Designer**: For instant feedback in the browser.
- **VS Code Extension**: For local preview without needing a binary.
- **Documentation Site**: For embedding interactive diagrams in documentation (like the one above!).

### CLI & CI/CD

The `sruja` CLI ([`cmd/sruja`](https://github.com/sruja-ai/sruja/tree/main/cmd/sruja)) is a static binary that wraps the core engine. It's designed for:

- **Local Development**: `sruja fmt`, `sruja lint`, `sruja compile`.
- **CI/CD**: `sruja score` to enforce architectural quality in your pipelines.
- **Export**: `sruja export json` to export architecture to JSON format.
- **Import**: `sruja import json` to import architecture from JSON format.
