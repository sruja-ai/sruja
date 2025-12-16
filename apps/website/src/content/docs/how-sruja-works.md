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

		container Designer "Sruja Designer" {
			technology "React/Vite"
			description "Interactive architecture design tool (apps/playground)"
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

		Designer -> WASM "uses for parsing/rendering"

		Website -> Designer "embeds"
	}

	User -> Sruja.CLI "runs commands"
	User -> Sruja.VSCode "writes DSL"
	User -> Sruja.Designer "designs architecture"
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
  story DesignerStory "Using Sruja Designer" {
    User -> Sruja.Website "visits designer"
    Sruja.Website -> Sruja.Designer "initializes"
    Sruja.Designer -> Sruja.WASM "loads engine"
    Sruja.Designer -> Sruja.WASM "parses DSL"
    Sruja.Designer -> User "renders interactive diagram"
  }

  story DesignerExports "Exporting from Designer" {
    User -> Sruja.Designer "clicks export"
    Sruja.Designer -> Sruja.WASM "requests JSON"
    Sruja.WASM -> Sruja.Designer "returns JSON data"
    Sruja.Designer -> User "downloads file"
  }

  // Designer-specific Scenarios
  scenario DesignerShare "Share From Designer" {
    User -> Sruja.Designer "edits DSL code"
    Sruja.Designer -> Browser "updates URL with code"
    User -> Sruja.Designer "clicks Share button"
    Sruja.Designer -> Browser "copies shareable URL"
    User -> User "shares URL with team"
  }

  scenario DesignerFormatPreview "Preview Multiple Formats" {
    User -> Sruja.Designer "loads architecture"
    User -> Sruja.Designer "switches to JSON preview"
    Sruja.Designer -> Sruja.WASM "parses DSL to JSON"
    Sruja.Designer -> User "displays JSON export"
  }

  scenario DesignerResizePanels "Resize Editor and Preview" {
    User -> Sruja.Designer "opens split view"
    Sruja.Designer -> User "shows editor and preview side-by-side"
    User -> Sruja.Designer "drags resize handle"
    Sruja.Designer -> User "adjusts panel sizes"
    Sruja.Designer -> Browser "saves panel preference"
  }

  scenario DesignerLoadExample "Load Example Architecture" {
    User -> Sruja.Designer "opens examples dropdown"
    Sruja.Designer -> User "shows available examples"
    User -> Sruja.Designer "selects example"
    Sruja.Designer -> Sruja.Website "loads example file"
    Sruja.Website -> Sruja.Designer "returns DSL content"
    Sruja.Designer -> Sruja.WASM "parses DSL"
    Sruja.Designer -> User "displays architecture"
    Sruja.Designer -> Browser "updates URL with code"
  }

  scenario DesignerURLState "URL State Management" {
    User -> Sruja.Designer "edits DSL code" [Input]
    Sruja.Designer -> Browser "debounces URL update" [Timeout]
    Browser -> Browser "updates URL hash with code" [HistoryAPI]
    User -> Browser "refreshes page" [Event]
    Browser -> Sruja.Designer "loads code from URL" [HashParse]
    Sruja.Designer -> Sruja.WASM "parses DSL from URL" [FunctionCall]
    Sruja.Designer -> User "restores architecture state" [DOM]
  }

  story DesignerEditingStory "Visual Editing in Designer" {
    User -> Sruja.Designer "opens editor"
    Sruja.Designer -> Sruja.WASM "loads engine"
    User -> Sruja.Designer "types DSL code"
    Sruja.Designer -> Sruja.WASM "validates code"
    Sruja.WASM -> Sruja.Designer "returns diagnostics"
    Sruja.Designer -> User "shows errors/diagram"
  }

  scenario DesignerAutosave "Autosave on Close" {
    Sruja.Designer -> Browser.LocalStore "save DSL snapshot"
    User -> Sruja.Designer "closes tab"
    User -> Sruja.Designer "reopens designer"
    Sruja.Designer -> Browser.LocalStore "load DSL snapshot"
    Sruja.Designer -> User "restores session"
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

- **Sruja Designer**: For instant feedback in the browser.
- **VS Code Extension**: For local preview without needing a binary.
- **Documentation Site**: For embedding interactive diagrams in documentation (like the one above!).

### CLI & CI/CD

The `sruja` CLI ([`cmd/sruja`](https://github.com/sruja-ai/sruja/tree/main/cmd/sruja)) is a static binary that wraps the core engine. It's designed for:

- **Local Development**: `sruja fmt`, `sruja lint`, `sruja compile`.
- **CI/CD**: `sruja score` to enforce architectural quality in your pipelines.
- **Export**: `sruja export json` to export architecture to JSON format.
- **Import**: `sruja import json` to import architecture from JSON format.
