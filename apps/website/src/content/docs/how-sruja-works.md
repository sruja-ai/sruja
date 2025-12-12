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
architecture "Sruja" {
	description "The Sruja Architecture-as-Code Platform"

	person User "Architect/Developer" {
		description "Uses Sruja to design and document systems"
	}

	system Sruja "Sruja Platform" {
		description "Tools for defining, visualizing, and analyzing software architecture"

		container CLI "Sruja CLI" {
			technology "Go"
			description "Command-line interface (cmd/sruja)"
		}

		container Engine "Core Engine" {
			technology "Go"
			description "Core logic for validation, scoring, and analysis (pkg/engine)"

			component Validation "Validation Engine" {
				technology "Go"
				description "Validates AST against rules (pkg/engine/rules)"
			}

			component Scorer "Scoring Engine" {
				technology "Go"
				description "Calculates architecture health score (pkg/engine/scorer.go)"
			}

			component Policy "Policy Engine" {
				technology "Go"
				description "Enforces custom policies (future: OPA/Rego)"
			}

			Scorer -> Validation "uses results from"
			Validation -> Policy "checks against"
		}

		container Language "Language Service" {
			technology "Go"
			description "Parser, AST, and LSP implementation (pkg/language)"
		}

		container WASM "WASM Module" {
			technology "Go/WASM"
			description "WebAssembly build of the core engine (cmd/wasm)"
		}

		container VSCode "VS Code Extension" {
			technology "TypeScript"
			description "Editor extension (apps/vscode-extension)"
		}

		container Playground "Sruja Playground" {
			technology "React/Vite"
			description "Interactive playground for testing Sruja code (apps/playground)"
		}

		container Website "Documentation Site" {
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

		Playground -> WASM "uses for parsing/rendering"

		Website -> Playground "embeds"
	}

	User -> Sruja.CLI "runs commands"
	User -> Sruja.VSCode "writes DSL"
	User -> Sruja.Playground "visualizes architecture"
	User -> Sruja.Website "reads docs"

	system Browser "Web Browser" {
		description "User's web browser environment"
		metadata {
			tags ["external"]
		}
		datastore LocalStore "Local Storage"
	}

	// ADRs
	adr ADR001 "Use WASM for Client-Side Execution" {
		status "Accepted"
		context "We need to run validation and parsing in the browser and VS Code without a backend server."
		decision "Compile the Go core engine to WebAssembly."
		consequences "Ensures consistent logic across all platforms but increases build complexity."
	}

  // Deployment
  deployment Production "Production Environment" {
    node GitHubPages "GitHub Pages" {
      containerInstance Pages
    }
    node Marketplace "VS Code Marketplace" {
      containerInstance VSCode
    }
    node Releases "GitHub Releases" {
      containerInstance Releases
    }
  }

  system GitHub "GitHub Platform" {
    description "Source control, CI/CD, and hosting"
    container Actions "GitHub Actions" {
      technology "YAML/Node"
      description "CI/CD workflows"
    }
    container Pages "GitHub Pages" {
      technology "Static Hosting"
      description "Hosts documentation site"
    }
    container Releases "GitHub Releases" {
      technology "File Hosting"
      description "Hosts CLI binaries"
    }
    Actions -> Pages "deploys to"
    Actions -> Releases "publishes to"
  }


  User -> GitHub "pushes code to" [Git]


  // Component Stories
  story PlaygroundStory "Using the Playground" {
    User -> Sruja.Website "visits playground"
    Sruja.Website -> Sruja.Playground "initializes"
    Sruja.Playground -> Sruja.WASM "loads engine"
    Sruja.Playground -> Sruja.WASM "parses DSL"
    Sruja.Playground -> User "renders interactive diagram"
  }

  story PlaygroundExports "Exporting from Playground" {
    User -> Sruja.Playground "clicks export"
    Sruja.Playground -> Sruja.WASM "requests JSON"
    Sruja.WASM -> Sruja.Playground "returns JSON data"
    Sruja.Playground -> User "downloads file"
  }

  // Playground-specific Scenarios
  scenario PlaygroundShare "Share From Playground" {
    User -> Sruja.Playground "edits DSL code"
    Sruja.Playground -> Browser "updates URL with code"
    User -> Sruja.Playground "clicks Share button"
    Sruja.Playground -> Browser "copies shareable URL"
    User -> User "shares URL with team"
  }

  scenario PlaygroundFormatPreview "Preview Multiple Formats" {
    User -> Sruja.Playground "loads architecture"
    User -> Sruja.Playground "switches to JSON preview"
    Sruja.Playground -> Sruja.WASM "parses DSL to JSON"
    Sruja.Playground -> User "displays JSON export"
  }

  scenario PlaygroundResizePanels "Resize Editor and Preview" {
    User -> Sruja.Playground "opens split view"
    Sruja.Playground -> User "shows editor and preview side-by-side"
    User -> Sruja.Playground "drags resize handle"
    Sruja.Playground -> User "adjusts panel sizes"
    Sruja.Playground -> Browser "saves panel preference"
  }

  scenario PlaygroundLoadExample "Load Example Architecture" {
    User -> Sruja.Playground "opens examples dropdown"
    Sruja.Playground -> User "shows available examples"
    User -> Sruja.Playground "selects example"
    Sruja.Playground -> Sruja.Website "loads example file"
    Sruja.Website -> Sruja.Playground "returns DSL content"
    Sruja.Playground -> Sruja.WASM "parses DSL"
    Sruja.Playground -> User "displays architecture"
    Sruja.Playground -> Browser "updates URL with code"
  }

  scenario PlaygroundURLState "URL State Management" {
    User -> Sruja.Playground "edits DSL code" [Input]
    Sruja.Playground -> Browser "debounces URL update" [Timeout]
    Browser -> Browser "updates URL hash with code" [HistoryAPI]
    User -> Browser "refreshes page" [Event]
    Browser -> Sruja.Playground "loads code from URL" [HashParse]
    Sruja.Playground -> Sruja.WASM "parses DSL from URL" [FunctionCall]
    Sruja.Playground -> User "restores architecture state" [DOM]
  }

  story PlaygroundEditingStory "Visual Editing in Playground" {
    User -> Sruja.Playground "opens editor"
    Sruja.Playground -> Sruja.WASM "loads engine"
    User -> Sruja.Playground "types DSL code"
    Sruja.Playground -> Sruja.WASM "validates code"
    Sruja.WASM -> Sruja.Playground "returns diagnostics"
    Sruja.Playground -> User "shows errors/diagram"
  }

  scenario PlaygroundAutosave "Autosave on Close" {
    Sruja.Playground -> Browser.LocalStore "save DSL snapshot"
    User -> Sruja.Playground "closes tab"
    User -> Sruja.Playground "reopens playground"
    Sruja.Playground -> Browser.LocalStore "load DSL snapshot"
    Sruja.Playground -> User "restores session"
  }

  story DocsStory "Reading Documentation" {
    User -> Sruja.Website "navigates to page"
    Sruja.Website -> GitHub.Pages "serves content"
    Sruja.Website -> User "displays text & code blocks"
  }

  // Specific Scenarios (The Behavior)
  scenario CIDev "Continuous Integration (Dev)" {
    User -> GitHub "pushes to main"
    GitHub -> GitHub.Actions "triggers CI"
    GitHub.Actions -> Sruja "builds & tests"
    GitHub.Actions -> GitHub.Pages "deploys dev site"
  }

  scenario ReleaseStaging "Release Candidate (Staging)" {
    User -> GitHub.Actions "dispatches release workflow"
    GitHub.Actions -> GitHub "creates release/<semVer> branch"
    GitHub.Actions -> GitHub "creates PR to prod"
    GitHub.Actions -> GitHub "creates <semVer>-RC1 tag"
    GitHub.Actions -> Sruja.VSCode "publishes pre-release extension"
  }

  scenario ReleaseFixes "Release Candidate Fixes" {
    User -> GitHub "commits fix to release/<semVer>"
    GitHub -> GitHub.Actions "triggers CI"
    GitHub.Actions -> GitHub "creates <semVer>-RC2 tag"
    GitHub.Actions -> Sruja.VSCode "updates pre-release extension"
  }

  scenario ReleaseProd "Production Release" {
    User -> GitHub "merges PR to prod"
    GitHub -> GitHub.Actions "triggers release"
    GitHub.Actions -> GitHub.Pages "deploys prod site"
    GitHub.Actions -> Sruja.VSCode "publishes extension"
    GitHub.Actions -> GitHub.Releases "publishes CLI binaries"
  }

  scenario HotfixProd "Hotfix to Production" {
    User -> GitHub.Actions "dispatches hotfix workflow"
    GitHub.Actions -> GitHub "creates hotfix branch"
    User -> GitHub "merges hotfix to prod"
    GitHub.Actions -> GitHub.Pages "deploys prod site"
    GitHub.Actions -> Sruja.VSCode "publishes patch update"
  }
}
```

## Key Components

### Core Engine (Go)

The [`pkg/engine`](https://github.com/sruja-ai/sruja/tree/main/pkg/engine) and [`pkg/language`](https://github.com/sruja-ai/sruja/tree/main/pkg/language) packages form the foundation. They define the DSL grammar, parse input files into an AST (Abstract Syntax Tree), and run validation rules (like cycle detection and layer enforcement).

### WebAssembly (WASM)

To ensure a consistent experience across all tools, we compile the Go engine to WebAssembly ([`cmd/wasm`](https://github.com/sruja-ai/sruja/tree/main/cmd/wasm)). This allows the **exact same parsing and validation logic** to run in:

- **Sruja Playground**: For instant feedback in the browser.
- **VS Code Extension**: For local preview without needing a binary.
- **Documentation Site**: For embedding interactive diagrams in documentation (like the one above!).

### CLI & CI/CD

The `sruja` CLI ([`cmd/sruja`](https://github.com/sruja-ai/sruja/tree/main/cmd/sruja)) is a static binary that wraps the core engine. It's designed for:

- **Local Development**: `sruja fmt`, `sruja lint`, `sruja compile`.
- **CI/CD**: `sruja score` to enforce architectural quality in your pipelines.
- **Export**: `sruja export json` to export architecture to JSON format.
- **Import**: `sruja import json` to import architecture from JSON format.
