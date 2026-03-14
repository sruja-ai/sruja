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
  constructor(start: Position, end: Position) {
    this.start = start;
    this.end = end;
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
    return new Uri('file', path);
  }
  static parse(value: string): Uri {
    return new Uri('file', value);
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

export class DiagnosticCollection {
  set(_uri: Uri, _diagnostics: Diagnostic[]): void {}
  delete(_uri: Uri): void {}
  get(_uri: Uri): Diagnostic[] | undefined { return undefined; }
}

export const languages = {
  registerDefinitionProvider: () => {},
  registerHoverProvider: () => {},
  registerDocumentSymbolProvider: () => {},
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
  subscriptions: unknown[] = [];
}
