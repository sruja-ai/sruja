export class Position {
  line: number;
  character: number;
  constructor(line: number, character: number) {
    this.line = line;
    this.character = character;
  }
}

export class Range {
  start: Position;
  end: Position;
  constructor(
    startOrLine: Position | number,
    endOrStartChar?: Position | number,
    endLine?: number,
  endChar?: number
  ) {
    if (typeof startOrLine === "number" && typeof endOrStartChar === "number") {
      this.start = new Position(startOrLine, endOrStartChar);
      this.end = new Position(endLine ?? startOrLine, endChar ?? endOrStartChar);
    } else {
      this.start = startOrLine as Position;
      this.end = endOrStartChar as Position;
    }
  }
}

export class MarkdownString {
  private content = '';
  appendMarkdown(value: string): void {
    this.content += value;
  }
}

export class Hover {
  constructor(public contents: MarkdownString, public range?: Range) {}
}

export class Location {
  constructor(public uri: Uri, public range: Range) {}
}

export class Uri {
  fsPath: string;
  constructor(public scheme: string, path: string) {
    this.fsPath = path;
  }
  static file(path: string): Uri {
    return new Uri("file", path);
  }
  static parse(value: string): Uri {
    if (value.startsWith("file://")) return Uri.file(value.slice(7));
    return new Uri("file", value);
  }
  static joinPath(base: Uri, ...pathSegments: string[]): Uri {
    const segments = [base.fsPath, ...pathSegments].filter(Boolean);
    return Uri.file(segments.join("/"));
  }
  toString(): string {
    if (this.scheme === "file") return "file://" + (this.fsPath.startsWith("/") ? this.fsPath : "/" + this.fsPath);
    return this.scheme + "://" + this.fsPath;
  }
}

export enum SymbolKind {
  Class = 4,
  Interface = 7,
  Method = 5,
  Function = 1,
  Boolean = 15,
  Enum = 12,
  Event = 13,
  Object = 18,
}

export class DocumentSymbol {
  constructor(
    public name: string,
    public detail: string,
    public kind: SymbolKind,
    public range: Range,
    public selectionRange: Range
  ) {}
}

export enum DiagnosticSeverity {
  Error = 0,
  Warning = 1,
  Information = 2,
  Hint = 3,
}

export class Diagnostic {
  range: Range;
  message: string;
  severity: DiagnosticSeverity;
  code?: string;
  source?: string;
  constructor(range: Range, message: string, severity?: DiagnosticSeverity) {
    this.range = range;
    this.message = message;
    this.severity = severity ?? DiagnosticSeverity.Error;
  }
}

export class WorkspaceEdit {
  insert(_uri: Uri, _position: Position, _text: string): void {}
}

export class CodeAction {
  diagnostics?: Diagnostic[];
  edit?: WorkspaceEdit;
  constructor(public title: string, public kind?: CodeActionKind) {}
}

export class CodeActionKind {
  static QuickFix = new CodeActionKind("quickfix");
  constructor(public value: string) {}
}

export class DiagnosticCollection {
  set(_uri: Uri, _diagnostics: Diagnostic[]): void {}
  delete(_uri: Uri): void {}
  get(_uri: Uri): Diagnostic[] | undefined { return undefined; }
}

export const languages = {
  registerDefinitionProvider: () => {},
  registerHoverProvider: () => {},
  registerDocumentSymbolProvider: () => {},
  registerCodeActionsProvider: () => {},
  createDiagnosticCollection: () => new DiagnosticCollection(),
};

export const workspace = {
  onDidOpenTextDocument: () => {},
  onDidSaveTextDocument: () => {},
  onDidChangeTextDocument: () => {},
  onDidCloseTextDocument: () => {},
  textDocuments: [],
  workspaceFolders: [],
  getConfiguration: () => ({ get: () => '' }),
  fs: {
    readFile: async () => Buffer.from(''),
    writeFile: async () => {},
  },
};

export const window = {
  createOutputChannel: () => ({ append: () => {}, appendLine: () => {}, clear: () => {}, show: () => {} }),
  showInformationMessage: async () => {},
  showWarningMessage: async () => {},
  showErrorMessage: async () => {},
  showQuickPick: async () => undefined,
  showTextDocument: async () => {},
  createWebviewPanel: () => ({ webview: { html: '' } }),
  activeTextEditor: undefined,
};

export const commands = {
  registerCommand: () => {},
  executeCommand: async () => {},
};

export class CancellationToken {
  isCancellationRequested = false;
  onCancellationRequested = () => ({ dispose: () => {} });
}

export class ExtensionContext {
  extensionPath = '';
  extensionUri = Uri.file('');
  subscriptions: unknown[] = [];
}

export enum TreeItemCollapsibleState {
  None = 0,
  Collapsed = 1,
  Expanded = 2,
}

export class ThemeIcon {
  constructor(public id: string) {}
}

export class TreeItem {
  label: string;
  collapsibleState: TreeItemCollapsibleState;
  command?: { command: string; title: string; arguments?: unknown[] };
  resourceUri?: Uri;
  iconPath?: ThemeIcon;
  constructor(label: string, collapsibleState?: TreeItemCollapsibleState) {
    this.label = label;
    this.collapsibleState = collapsibleState ?? TreeItemCollapsibleState.None;
  }
}

export class EventEmitter<T> {
  fire(_data?: T): void {}
  get event(): { (_listener: (e: T) => unknown): { dispose(): void } } {
    return () => ({ dispose: () => {} });
  }
}
