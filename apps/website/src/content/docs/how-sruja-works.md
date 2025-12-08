---
title: "How Sruja Works"
weight: 2
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

		container Studio "Sruja Studio" {
			technology "React/Vite"
			description "Web-based editor and visualizer (apps/studio-core)"
		}

		container Viewer "Sruja Viewer" {
			technology "React"
			description "Embeddable viewer for Sruja diagrams (packages/viewer)"
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
		VSCode -> WASM "uses for preview"

		Studio -> WASM "uses for parsing/rendering"
		
		Website -> Viewer "embeds"
	}

User -> Sruja.CLI "runs commands"
User -> Sruja.VSCode "writes DSL"
User -> Sruja.Studio "visualizes architecture"
User -> Sruja.Website "reads docs"

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

    GitHub.Actions -> GitHub.Pages "deploys to"
    GitHub.Actions -> GitHub.Releases "publishes to"
  }

  User -> GitHub "pushes code to" [Git]


  // Component Stories
  story ViewerStory "Viewing Embedded Diagrams" {
    User -> Sruja.Website "visits documentation"
    Sruja.Website -> Sruja.Viewer "initializes"
    Sruja.Viewer -> Sruja.WASM "loads engine"
    Sruja.Viewer -> Sruja.WASM "parses DSL"
    Sruja.Viewer -> User "renders interactive diagram"
  }

  story ViewerExports "Exporting Diagrams" {
    User -> Sruja.Viewer "clicks export"
    Sruja.Viewer -> Sruja.WASM "requests SVG/PNG"
    Sruja.WASM -> Sruja.Viewer "returns image data"
    Sruja.Viewer -> User "downloads file"
  }

  // Viewer-specific Scenarios
  scenario ViewerShare "Share From Viewer" {
    User -> Sruja.Viewer "edits DSL code"
    Sruja.Viewer -> Browser "updates URL with code"
    User -> Sruja.Viewer "clicks Share button"
    Sruja.Viewer -> Browser "copies shareable URL"
    User -> User "shares URL with team"
  }

  scenario ViewerOpenStudio "Open In Studio From Viewer" {
    User -> Sruja.Viewer "views architecture"
    User -> Sruja.Viewer "clicks Open in Studio"
    Sruja.Viewer -> Sruja.Studio "opens with current DSL"
    Sruja.Studio -> User "loads architecture for editing"
  }

  scenario ViewerFormatPreview "Preview Multiple Formats" {
    User -> Sruja.Viewer "loads architecture"
    User -> Sruja.Viewer "switches to JSON preview"
    Sruja.Viewer -> Sruja.WASM "parses DSL to JSON"
    Sruja.Viewer -> User "displays JSON export"
    User -> Sruja.Viewer "switches to Markdown preview"
    Sruja.Viewer -> Sruja.WASM "generates markdown"
    Sruja.Viewer -> User "displays markdown with diagrams"
    User -> Sruja.Viewer "switches to PDF preview"
    Sruja.Viewer -> Sruja.WASM "generates markdown"
    Sruja.Viewer -> Browser "generates PDF"
    Sruja.Viewer -> User "displays PDF preview"
  }

  scenario ViewerResizePanels "Resize Editor and Preview" {
    User -> Sruja.Viewer "opens split view"
    Sruja.Viewer -> User "shows editor and preview side-by-side"
    User -> Sruja.Viewer "drags resize handle"
    Sruja.Viewer -> User "adjusts panel sizes"
    Sruja.Viewer -> Browser "saves panel preference"
  }

  scenario ViewerLoadExample "Load Example Architecture" {
    User -> Sruja.Viewer "opens examples dropdown"
    Sruja.Viewer -> User "shows available examples"
    User -> Sruja.Viewer "selects example"
    Sruja.Viewer -> Sruja.Website "loads example file"
    Sruja.Website -> Sruja.Viewer "returns DSL content"
    Sruja.Viewer -> Sruja.WASM "parses DSL"
    Sruja.Viewer -> User "displays architecture"
    Sruja.Viewer -> Browser "updates URL with code"
  }

  scenario ViewerURLState "URL State Management" {
    User -> Viewer "edits DSL code" [Input]
    Viewer -> Browser "debounces URL update" [Timeout]
    Browser -> Browser "updates URL hash with code" [HistoryAPI]
    User -> Browser "refreshes page" [Event]
    Browser -> Viewer "loads code from URL" [HashParse]
    Viewer -> WASM "parses DSL from URL" [FunctionCall]
    Viewer -> User "restores architecture state" [DOM]
  }

  story StudioStory "Visual Editing in Studio" {
    User -> Sruja.Studio "opens editor"
    Sruja.Studio -> Sruja.WASM "loads engine"
    User -> Sruja.Studio "types DSL code"
    Sruja.Studio -> Sruja.WASM "validates code"
    Sruja.WASM -> Sruja.Studio "returns diagnostics"
    Sruja.Studio -> User "shows errors/diagram"
  }

  // Studio-specific Scenarios
  scenario StudioHandoff "Open In Viewer" {
    User -> Sruja.Studio "clicks View in Viewer"
    Sruja.Studio -> Sruja.Viewer "opens with current DSL"
    Sruja.Viewer -> User "interactive exploration"
  }

  scenario StudioAutosave "Autosave on Close" {
    system Browser {
      datastore LocalStore "Local Storage"
    }
    Studio -> LocalStore "save DSL snapshot"
    User -> Studio "closes tab"
    User -> Studio "reopens Studio"
    Studio -> LocalStore "load DSL snapshot"
    Studio -> User "restores session"
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
    GitHub.Actions -> Marketplace "publishes pre-release extension"
  }

  scenario ReleaseFixes "Release Candidate Fixes" {
    User -> GitHub "commits fix to release/<semVer>"
    GitHub -> GitHub.Actions "triggers CI"
    GitHub.Actions -> GitHub "creates <semVer>-RC2 tag"
    GitHub.Actions -> Marketplace "updates pre-release extension"
  }

  scenario ReleaseProd "Production Release" {
    User -> GitHub "merges PR to prod"
    GitHub -> GitHub.Actions "triggers release"
    GitHub.Actions -> GitHub.Pages "deploys prod site"
    GitHub.Actions -> Marketplace "publishes extension"
    GitHub.Actions -> GitHub.Releases "publishes CLI binaries"
  }

  scenario HotfixProd "Hotfix to Production" {
    User -> GitHub.Actions "dispatches hotfix workflow"
    GitHub.Actions -> GitHub "creates hotfix branch"
    User -> GitHub "merges hotfix to prod"
    GitHub.Actions -> GitHub.Pages "deploys prod site"
    GitHub.Actions -> Marketplace "publishes patch update"
  }
}
```

## Key Components

### Core Engine (Go)
The [`pkg/engine`](https://github.com/sruja-ai/sruja/tree/main/pkg/engine) and [`pkg/language`](https://github.com/sruja-ai/sruja/tree/main/pkg/language) packages form the foundation. They define the DSL grammar, parse input files into an AST (Abstract Syntax Tree), and run validation rules (like cycle detection and layer enforcement).

### WebAssembly (WASM)
To ensure a consistent experience across all tools, we compile the Go engine to WebAssembly ([`cmd/wasm`](https://github.com/sruja-ai/sruja/tree/main/cmd/wasm)). This allows the **exact same parsing and validation logic** to run in:
-   **Sruja Studio**: For instant feedback in the browser.
-   **VS Code Extension**: For local preview without needing a binary.
-   **Sruja Viewer**: For embedding diagrams in documentation (like the one above!).

### CLI & CI/CD
The `sruja` CLI ([`cmd/sruja`](https://github.com/sruja-ai/sruja/tree/main/cmd/sruja)) is a static binary that wraps the core engine. It's designed for:
-   **Local Development**: `sruja fmt`, `sruja validate`.
-   **CI/CD**: `sruja score` to enforce architectural quality in your pipelines.
