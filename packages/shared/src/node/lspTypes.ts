// packages/shared/src/node/lspTypes.ts
// LSP (Language Server Protocol) type definitions for Node.js WASM adapter

/**
 * Diagnostic information from language server.
 * 
 * @public
 */
export interface Diagnostic {
  code: string;
  severity: "Error" | "Warning" | "Info";
  message: string;
  location: {
    file: string;
    line: number;
    column: number;
  };
}

/**
 * Hover information for a symbol at a position.
 * 
 * @public
 */
export interface HoverInfo {
  contents: string;
}

/**
 * Completion item for code completion.
 * 
 * @public
 */
export interface CompletionItem {
  label: string;
  kind: string;
}

/**
 * Location in source code (file, line, column).
 * 
 * @public
 */
export interface Location {
  file: string;
  line: number;
  column: number;
}

/**
 * Symbol information.
 * 
 * @public
 */
export interface Symbol {
  name: string;
  kind: string;
  line: number;
}

/**
 * Code action for fixing issues.
 * 
 * @public
 */
export interface CodeAction {
  title: string;
  command: string;
  arguments?: unknown[];
}

/**
 * Document link (e.g., to external documentation).
 * 
 * @public
 */
export interface DocumentLink {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  target?: string;
  tooltip?: string;
}

/**
 * Folding range for code folding.
 * 
 * @public
 */
export interface FoldingRange {
  startLine: number;
  startCharacter?: number;
  endLine: number;
  endCharacter?: number;
  kind?: string;
}

