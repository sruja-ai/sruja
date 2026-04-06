## 1. Architecture Design

```mermaid
graph TD
    A[VS Code Editor] --> B[VS Code Extension Host]
    B --> C[Language Client]
    C --> D[Language Server Protocol]
    D --> E[sruja lsp Server]
    
    subgraph "VS Code Environment"
        A
        B
    end
    
    subgraph "Extension Layer"
        C
    end
    
    subgraph "Language Server"
        E
    end
```

## 2. Technology Description

* **Frontend**: TypeScript + VS Code Extension API

* **Initialization Tool**: yo code (VS Code Extension Generator)

* **Backend**: None (uses external `sruja lsp` server)

* **Communication**: Language Server Protocol (stdio)

## 3. Route Definitions

| Route                    | Purpose                                |
| ------------------------ | -------------------------------------- |
| extension.activate       | Triggered when .sruja files are opened |
| extension.formatDocument | Format entire Sruja document           |
| extension.goToDefinition | Navigate to symbol definition          |
| extension.findReferences | Find all symbol references             |
| extension.restartServer  | Restart LSP server connection          |

## 4. API Definitions

### 4.1 Core Extension APIs

**Language Server Configuration**

```typescript
interface LanguageServerConfig {
    command: string;        // Path to sruja lsp executable
    args: string[];         // Additional server arguments
    fileTypes: string[];    // ['.sruja']
    initializationOptions: object;
}
```

**Extension Settings**

```typescript
interface ExtensionSettings {
    srujaLanguageServer: {
        path: string;           // Custom LSP server path
        enableLogging: boolean;
        logLevel: 'error' | 'warn' | 'info' | 'debug';
    };
    formatting: {
        enabled: boolean;
        tabSize: number;
        insertSpaces: boolean;
    };
}
```

## 5. Server Architecture Diagram

```mermaid
graph TD
    A[VS Code Extension] --> B[Language Client]
    B --> C[JSON-RPC Protocol]
    C --> D[LSP Server Process]
    D --> E[stdio Communication]
    
    subgraph "Extension Host"
        A
        B
    end
    
    subgraph "External Process"
        D
    end
```

## 6. Data Model

### 6.1 Extension State Management

```mermaid
classDiagram
    class ExtensionContext {
        +subscriptions: Disposable[]
        +workspaceState: Memento
        +globalState: Memento
        +extensionPath: string
        +storagePath: string
    }
    
    class LanguageClient {
        +id: string
        +name: string
        +serverOptions: ServerOptions
        +clientOptions: LanguageClientOptions
        +start(): void
        +stop(): void
    }
    
    class ServerStatus {
        +state: 'running' | 'stopped' | 'error'
        +message: string
        +timestamp: Date
    }
    
    ExtensionContext --> LanguageClient : manages
    LanguageClient --> ServerStatus : reports
```

### 6.2 Key Components

**Extension Manifest (package.json)**

```json
{
  "name": "sruja-language-support",
  "displayName": "Sruja DSL Language Support",
  "activationEvents": ["onLanguage:sruja"],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [{
      "id": "sruja",
      "extensions": [".sruja"],
      "configuration": "./language-configuration.json"
    }],
    "configuration": {
      "title": "Sruja Language Server",
      "properties": {
        "srujaLanguageServer.path": {
          "type": "string",
          "default": "sruja-lsp",
          "description": "Path to sruja language server executable"
        }
      }
    }
  }
}
```

**Language Configuration**

```json
{
  "comments": {
    "lineComment": "//",
    "blockComment": ["/*", "*/"]
  },
  "brackets": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"]
  ],
  "autoClosingPairs": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
    ["\"", "\""],
    ["'", "'"]
  ]
}
```

