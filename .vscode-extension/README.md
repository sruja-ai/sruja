# Sruja Language Support for VS Code

Syntax highlighting and snippets for the Sruja architecture-as-code language.

## Features

- **Syntax Highlighting**: Full syntax highlighting for `.sruja` files
- **Snippets**: Quick snippets for common patterns
- **Auto-closing**: Brackets, quotes, and braces

## Snippets

- `workspace` - Create a workspace block
- `system` - Create a system
- `container` - Create a container
- `relation` - Create a relation
- `requirements` - Create a requirements block
- `adrs` - Create an ADRs block
- `!workspace` - Create a full example workspace

## Installation

### From Source

1. Copy the `.vscode-extension` directory to your VS Code extensions folder:
   ```bash
   cp -r .vscode-extension ~/.vscode/extensions/sruja
   ```

2. Reload VS Code

### From Marketplace (Coming Soon)

Search for "Sruja" in the VS Code Extensions marketplace.

## Usage

Create a file with `.sruja` extension and start typing! Use snippets by typing the prefix and pressing Tab.

## Example

```sruja
workspace {
  model {
    system User "End User"
    system API "API Service"
    User -> API "calls"
  }
}
```

## Links

- [GitHub](https://github.com/sruja-ai/sruja)
- [Documentation](https://github.com/sruja-ai/sruja/blob/main/README.md)
