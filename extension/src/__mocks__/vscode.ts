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

  contains(pos: Position): boolean {
    const afterStart =
      pos.line > this.start.line || (pos.line === this.start.line && pos.character >= this.start.character);
    const beforeEnd =
      pos.line < this.end.line || (pos.line === this.end.line && pos.character <= this.end.character);
    return afterStart && beforeEnd;
  }
}

export class Selection extends Range {
  anchor: Position;
  active: Position;
  constructor(anchor: Position, active: Position) {
    super(anchor, active);
    this.anchor = anchor;
    this.active = active;
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
  public operations: Array<
    | { type: "insert"; uri: Uri; position: Position; text: string }
    | { type: "replace"; uri: Uri; range: Range; text: string }
  > = [];

  insert(uri: Uri, position: Position, text: string): void {
    this.operations.push({ type: "insert", uri, position, text });
  }

  replace(uri: Uri, range: Range, text: string): void {
    this.operations.push({ type: "replace", uri, range, text });
  }
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

export class CodeLens {
  constructor(public range: Range, public command?: { title: string; command: string; arguments?: unknown[] }) {}
}

export enum CompletionItemKind {
  Text = 0,
  Method = 1,
  Function = 2,
  Constructor = 3,
  Field = 4,
  Variable = 5,
  Class = 6,
  Interface = 7,
  Module = 8,
  Property = 9,
  Unit = 10,
  Value = 11,
  Enum = 12,
  Keyword = 13,
}

export class CompletionItem {
  detail?: string;
  documentation?: string;
  constructor(public label: string, public kind?: CompletionItemKind) {}
}

export class TextEdit {
  constructor(public range: Range, public newText: string) {}
  static replace(range: Range, newText: string): TextEdit {
    return new TextEdit(range, newText);
  }
}

export const languages = {
  registerDefinitionProvider: () => {},
  registerHoverProvider: () => {},
  registerDocumentSymbolProvider: () => {},
  registerCodeActionsProvider: () => {},
  registerCodeLensProvider: () => {},
  registerCompletionItemProvider: () => {},
  registerRenameProvider: () => {},
  registerReferenceProvider: () => {},
  registerDocumentFormattingEditProvider: () => {},
  createDiagnosticCollection: () => new DiagnosticCollection(),
  getDiagnostics: (_uri?: Uri) => [],
};

export const workspace = {
  onDidOpenTextDocument: () => ({ dispose: () => {} }),
  onDidSaveTextDocument: () => ({ dispose: () => {} }),
  onDidChangeTextDocument: () => ({ dispose: () => {} }),
  onDidCloseTextDocument: () => ({ dispose: () => {} }),
  textDocuments: [],
  workspaceFolders: [],
  getConfiguration: () => ({ get: () => '' }),
  getWorkspaceFolder: (_uri: Uri) => undefined,
  findFiles: async (_include: string, _exclude?: string) => [],
  openTextDocument: async (arg: any) => {
    const computePositionAt = (text: string, offset: number): Position => {
      const clipped = Math.max(0, Math.min(text.length, offset));
      const before = text.slice(0, clipped);
      const lines = before.split(/\r?\n/);
      const line = Math.max(0, lines.length - 1);
      const character = lines[line]?.length ?? 0;
      return new Position(line, character);
    };

    if (arg && typeof arg === "object" && "content" in arg) {
      const content = String((arg as any).content ?? "");
      const languageId = String((arg as any).language ?? "plaintext");
      const uri = Uri.file("/untitled");
      return {
        uri,
        languageId,
        version: 1,
        lineCount: content.split(/\r?\n/).length,
        getText: () => content,
        lineAt: (line: number) => {
          const lines = content.split(/\r?\n/);
          const text = lines[line] ?? "";
          const start = new Position(line, 0);
          const end = new Position(line, text.length);
          return { text, range: new Range(start, end) };
        },
        getWordRangeAtPosition: () => undefined,
        positionAt: (offset: number) => computePositionAt(content, offset),
      };
    }

    const uri = arg as Uri;
    const content = "";
    return {
      uri,
      languageId: "plaintext",
      version: 1,
      lineCount: 0,
      getText: () => content,
      lineAt: (line: number) => {
        const start = new Position(line, 0);
        const end = new Position(line, 0);
        return { text: "", range: new Range(start, end) };
      },
      getWordRangeAtPosition: () => undefined,
      positionAt: (offset: number) => computePositionAt(content, offset),
    };
  },
  fs: {
    readFile: async () => new Uint8Array(),
    writeFile: async () => {},
    stat: async () => ({ size: 0, mtime: 0 }),
    createDirectory: async () => {},
  },
};

export const ProgressLocation = {
  Notification: 0,
};

export enum ViewColumn {
  One = 1,
  Beside = 2,
}

export enum TextEditorRevealType {
  InCenter = 0,
}

export const window = {
  createOutputChannel: () => ({ append: () => {}, appendLine: () => {}, clear: () => {}, show: () => {} }),
  showInformationMessage: async () => {},
  showWarningMessage: async () => {},
  showErrorMessage: async () => {},
  showQuickPick: async () => undefined,
  showTextDocument: async () => ({ selection: undefined, revealRange: () => {} }),
  withProgress: async (_options: unknown, task: (progress: unknown) => unknown) => task({ report: () => {} }),
  createWebviewPanel: () => {
    let onDispose: (() => void) | undefined;
    const panel: any = {
      webview: {
        html: "",
        options: {},
        cspSource: "vscode-resource:",
        asWebviewUri: (uri: any) => uri,
        onDidReceiveMessage: (handler: (message: any) => any) => {
          panel.__onDidReceiveMessage = handler;
        },
      },
      onDidDispose: (cb: () => void) => {
        onDispose = cb;
      },
      reveal: () => {},
      dispose: () => {
        onDispose?.();
      },
    };
    return panel;
  },
  visibleTextEditors: [],
  activeTextEditor: undefined,
};

export const commands = {
  registerCommand: () => {},
  executeCommand: async () => {},
};

export const env = {
  clipboard: {
    writeText: async () => {},
  },
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
