// apps/vscode-extension/src/wasmLspProvider/utils.ts
// Utility functions for WASM LSP provider

import * as vscode from "vscode";

// Performance optimization: Cache for parsed documents
export interface CachedDocument {
  text: string;
  diagnostics: import("@sruja/shared/node/wasmAdapter").Diagnostic[];
  symbols: import("@sruja/shared/node/wasmAdapter").Symbol[];
  timestamp: number;
}

export const documentCache = new Map<string, CachedDocument>();

export function getCacheTTL(): number {
  const config = vscode.workspace.getConfiguration("sruja.performance");
  return config.get<number>("cacheTTL", 5000);
}

export function isCachingEnabled(): boolean {
  const config = vscode.workspace.getConfiguration("sruja.performance");
  return config.get<boolean>("enableCaching", true);
}

export function getDebounceDelay(): number {
  const config = vscode.workspace.getConfiguration("sruja.performance");
  return config.get<number>("diagnosticsDebounce", 300);
}

// Debounce utility
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;
  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      func(...args);
    };
    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(later, wait);
  };
}

// Logging utility
let outputChannel: vscode.OutputChannel | null = null;

export function setOutputChannel(channel: vscode.OutputChannel) {
  outputChannel = channel;
}

export function getOutputChannel(): vscode.OutputChannel | null {
  return outputChannel;
}

export function log(message: string, type: "info" | "warn" | "error" = "info") {
  const timestamp = new Date().toISOString();
  const prefix = `[${timestamp}] [${type.toUpperCase()}]`;
  const fullMessage = `${prefix} ${message}`;

  // Log to console (for development)
  if (type === "error") {
    console.error(fullMessage);
  } else if (type === "warn") {
    console.warn(fullMessage);
  } else {
    console.log(fullMessage);
  }

  // Log to output channel (for user visibility)
  if (outputChannel) {
    outputChannel.appendLine(fullMessage);
  }
}

export function setupConsoleInterception() {
  if (!outputChannel) return;

  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;

  console.log = (...args: any[]) => {
    originalLog.apply(console, args);
    if (outputChannel && args.some((arg) => String(arg).includes("[WASM]"))) {
      outputChannel.appendLine(`[WASM] ${args.map((a) => String(a)).join(" ")}`);
    }
  };

  console.warn = (...args: any[]) => {
    originalWarn.apply(console, args);
    if (outputChannel && args.some((arg) => String(arg).includes("[WASM]"))) {
      outputChannel.appendLine(`[WASM] ${args.map((a) => String(a)).join(" ")}`);
    }
  };

  console.error = (...args: any[]) => {
    originalError.apply(console, args);
    if (outputChannel && args.some((arg) => String(arg).includes("[WASM]"))) {
      outputChannel.appendLine(`[WASM ERROR] ${args.map((a) => String(a)).join(" ")}`);
    }
  };
}

// Helper function to check if character is identifier character
export function isIdentChar(c: string): boolean {
  return /[a-zA-Z0-9_]/.test(c);
}

// Map symbol kind from Go to VS Code SymbolKind
export function mapSymbolKind(kind: string): vscode.SymbolKind {
  switch (kind.toLowerCase()) {
    case "system":
      return vscode.SymbolKind.Class;
    case "container":
      return vscode.SymbolKind.Module;
    case "component":
      return vscode.SymbolKind.Method;
    case "datastore":
      return vscode.SymbolKind.Variable; // Database not available in older VS Code versions
    case "person":
      return vscode.SymbolKind.Interface;
    default:
      return vscode.SymbolKind.Variable;
  }
}

