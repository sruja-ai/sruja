import * as vscode from "vscode";
import { ExtensionContext, CancellationToken } from "./__mocks__/vscode";
import { buildDefinitionLinks, SrujaDefinitionProvider, SrujaHoverProvider, SrujaDocumentSymbolProvider } from "./providers";
import { SrujaElement, SrujaDocumentSymbol } from "./wasm";

jest.mock("./wasm", () => {
  const actual = jest.requireActual<typeof import("./wasm")>("./wasm");
  return {
    ...actual,
    getDocumentSymbolsFromWasm: jest.fn(),
    getElementsFromWasm: jest.fn(),
  };
});

function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

describe('buildDefinitionLinks', () => {
  it('returns targetSelectionRange matching element range', () => {
    const elements: SrujaElement[] = [
      {
        id: 'User',
        kind: 'element',
        title: 'Test User',
        range: { start: { line: 0, character: 0 }, end: { line: 2, character: 1 } },
      },
    ];

    const wordRange = { start: { line: 0, character: 8 }, end: { line: 0, character: 12 } };

    const result = buildDefinitionLinks('User', wordRange, elements);

    expect(result).toBeDefined();
    expect(result).toHaveLength(1);

    const link = result![0];
    expect(link.targetSelectionRange).toEqual({
      start: { line: 0, character: 0 },
      end: { line: 2, character: 1 },
    });
    expect(link.targetRange).toEqual({
      start: { line: 0, character: 0 },
      end: { line: 2, character: 1 },
    });
    expect(link.originSelectionRange).toEqual(wordRange);
  });

  it('returns targetSelectionRange for nested elements', () => {
    const elements: SrujaElement[] = [
      {
        id: 'Payment.ProcessPayment',
        kind: 'flow',
        title: 'Process',
        range: { start: { line: 1, character: 2 }, end: { line: 3, character: 3 } },
      },
    ];

    const wordRange = { start: { line: 1, character: 7 }, end: { line: 1, character: 23 } };

    const result = buildDefinitionLinks('ProcessPayment', wordRange, elements);

    expect(result).toBeDefined();
    expect(result).toHaveLength(1);

    const link = result![0];
    expect(link.targetSelectionRange).toEqual({
      start: { line: 1, character: 2 },
      end: { line: 3, character: 3 },
    });
  });

  it('matches element by full id', () => {
    const elements: SrujaElement[] = [
      {
        id: 'Payment.ProcessPayment',
        kind: 'flow',
        title: 'Process',
        range: { start: { line: 1, character: 2 }, end: { line: 3, character: 3 } },
      },
    ];

    const wordRange = { start: { line: 0, character: 0 }, end: { line: 0, character: 25 } };

    const result = buildDefinitionLinks('Payment.ProcessPayment', wordRange, elements);

    expect(result).toBeDefined();
    expect(result).toHaveLength(1);
  });

  it('returns undefined when element not found', () => {
    const elements: SrujaElement[] = [];

    const wordRange = { start: { line: 0, character: 8 }, end: { line: 0, character: 12 } };

    const result = buildDefinitionLinks('NonExistent', wordRange, elements);

    expect(result).toBeUndefined();
  });

  it('returns undefined for empty word', () => {
    const elements: SrujaElement[] = [
      {
        id: 'User',
        kind: 'element',
        title: 'Test User',
        range: { start: { line: 0, character: 0 }, end: { line: 2, character: 1 } },
      },
    ];

    const wordRange = { start: { line: 0, character: 8 }, end: { line: 0, character: 8 } };

    const result = buildDefinitionLinks('', wordRange, elements);

    expect(result).toBeUndefined();
  });
});

describe("SrujaDefinitionProvider", () => {
  it("returns undefined for non-sruja language", async () => {
    const doc = {
      languageId: "plaintext",
      getText: () => "",
      uri: { fsPath: "/a.sruja" },
    } as vscode.TextDocument;
    const provider = new SrujaDefinitionProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDefinition(
      doc,
      new vscode.Position(0, 0),
      new CancellationToken()
    );
    expect(result).toBeUndefined();
  });

  it("returns location link when element found via WASM", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      {
        id: "App",
        kind: "element",
        title: "Application",
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } },
      } as SrujaElement,
    ]);
    const doc = {
      languageId: "sruja",
      getText: (range?: vscode.Range) => (range ? "App" : "system App {}"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => new vscode.Range(0, 7, 0, 10),
    } as unknown as vscode.TextDocument;
    const provider = new SrujaDefinitionProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDefinition(
      doc,
      new vscode.Position(0, 8),
      new CancellationToken()
    );
    expect(result).toHaveLength(1);
    expect(result![0].targetRange.start.line).toBe(0);
    expect(result![0].targetUri.fsPath).toContain("test.sruja");
  });

  it("returns undefined when getWordRangeAtPosition returns undefined", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      { id: "App", kind: "element", title: null, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } } } as SrujaElement,
    ]);
    const doc = {
      languageId: "sruja",
      getText: () => "system App {}",
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => undefined,
    } as unknown as vscode.TextDocument;
    const provider = new SrujaDefinitionProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDefinition(doc, new vscode.Position(0, 8), new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns undefined when elements is empty", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([]);
    const doc = {
      languageId: "sruja",
      getText: (range?: vscode.Range) => (range ? "App" : "system App {}"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => new vscode.Range(0, 7, 0, 10),
    } as unknown as vscode.TextDocument;
    const provider = new SrujaDefinitionProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDefinition(doc, new vscode.Position(0, 8), new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns undefined when getElementsFromWasm returns null", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue(null);
    const doc = {
      languageId: "sruja",
      getText: () => "App",
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => new vscode.Range(0, 0, 0, 3),
    } as unknown as vscode.TextDocument;
    const provider = new SrujaDefinitionProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDefinition(doc, new vscode.Position(0, 1), new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns undefined when word at position is empty", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      { id: "App", kind: "element", title: null, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } } } as SrujaElement,
    ]);
    const doc = {
      languageId: "sruja",
      getText: (range?: vscode.Range) => (range ? "   " : "system App {}"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => new vscode.Range(0, 7, 0, 10),
    } as unknown as vscode.TextDocument;
    const provider = new SrujaDefinitionProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDefinition(doc, new vscode.Position(0, 8), new CancellationToken());
    expect(result).toBeUndefined();
  });
});

describe("SrujaHoverProvider", () => {
  it("returns undefined for non-sruja language", async () => {
    const doc = {
      languageId: "plaintext",
      getText: () => "",
      uri: { fsPath: "/a.sruja" },
    } as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(
      doc,
      new vscode.Position(0, 0),
      new CancellationToken()
    );
    expect(result).toBeUndefined();
  });

  it("returns undefined when getElementsFromWasm returns null", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue(null);
    const doc = {
      languageId: "sruja",
      getText: () => "App",
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => new vscode.Range(0, 0, 0, 3),
    } as unknown as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(doc, new vscode.Position(0, 1), new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns undefined when getWordRangeAtPosition returns undefined", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([{ id: "App", kind: "element", title: null, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } } } as SrujaElement]);
    const doc = {
      languageId: "sruja",
      getText: () => "App",
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => undefined,
    } as unknown as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(doc, new vscode.Position(0, 5), new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns undefined when word at position is empty", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([{ id: "App", kind: "element", title: null, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } } } as SrujaElement]);
    const doc = {
      languageId: "sruja",
      getText: (r?: vscode.Range) => (r ? "   " : "App"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => new vscode.Range(0, 0, 0, 3),
    } as unknown as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(doc, new vscode.Position(0, 1), new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns hover for element with no title and flat id (no parent)", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      { id: "App", kind: "system", title: null, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } } } as SrujaElement,
    ]);
    const wordRange = new vscode.Range(0, 7, 0, 10);
    const doc = {
      languageId: "sruja",
      getText: (r?: vscode.Range) => (r ? "App" : "system App {}"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => wordRange,
    } as unknown as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(doc, new vscode.Position(0, 8), new CancellationToken());
    expect(result).toBeDefined();
    expect(result!.range).toEqual(wordRange);
  });

  it("returns hover when element found via WASM", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      {
        id: "Payment.Process",
        kind: "flow",
        title: "Process payment",
        range: { start: { line: 2, character: 2 }, end: { line: 4, character: 0 } },
      } as SrujaElement,
    ]);
    const wordRange = new vscode.Range(0, 9, 0, 16);
    const doc = {
      languageId: "sruja",
      getText: (r?: vscode.Range) => (r ? "Process" : "ref Payment.Process"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => wordRange,
    } as unknown as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(
      doc,
      new vscode.Position(0, 12),
      new CancellationToken()
    );
    expect(result).toBeDefined();
    expect(result!.contents).toBeDefined();
  });

  it("returns undefined when no element matches word", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      { id: "Other", kind: "element", title: null, range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } } } as SrujaElement,
    ]);
    const wordRange = new vscode.Range(0, 0, 0, 3);
    const doc = {
      languageId: "sruja",
      getText: (r?: vscode.Range) => (r ? "App" : "App"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => wordRange,
    } as unknown as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(doc, new vscode.Position(0, 1), new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns hover with title and parent for nested element", async () => {
    const { getElementsFromWasm } = await import("./wasm");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      {
        id: "Sys.Container",
        kind: "container",
        title: "My Container",
        range: { start: { line: 1, character: 0 }, end: { line: 3, character: 0 } },
      } as SrujaElement,
    ]);
    const wordRange = new vscode.Range(0, 4, 0, 13);
    const doc = {
      languageId: "sruja",
      getText: (r?: vscode.Range) => (r ? "Container" : "ref Sys.Container"),
      uri: vscode.Uri.file("/test.sruja"),
      getWordRangeAtPosition: () => wordRange,
    } as unknown as vscode.TextDocument;
    const provider = new SrujaHoverProvider(asContext(new ExtensionContext()));
    const result = await provider.provideHover(doc, new vscode.Position(0, 8), new CancellationToken());
    expect(result).toBeDefined();
    expect(result!.range).toEqual(wordRange);
  });
});

describe("SrujaDocumentSymbolProvider", () => {
  it("returns undefined for non-sruja language", async () => {
    const doc = {
      languageId: "plaintext",
      getText: () => "",
      uri: { fsPath: "/a.sruja" },
    } as vscode.TextDocument;
    const provider = new SrujaDocumentSymbolProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDocumentSymbols(doc, new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns undefined when getDocumentSymbolsFromWasm returns null", async () => {
    const { getDocumentSymbolsFromWasm } = await import("./wasm");
    (getDocumentSymbolsFromWasm as jest.Mock).mockResolvedValue(null);
    const doc = {
      languageId: "sruja",
      getText: () => "system App {}",
      uri: { fsPath: "/test.sruja" },
    } as vscode.TextDocument;
    const provider = new SrujaDocumentSymbolProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDocumentSymbols(doc, new CancellationToken());
    expect(result).toBeUndefined();
  });

  it("returns symbols from WASM and maps kinds", async () => {
    const { getDocumentSymbolsFromWasm } = await import("./wasm");
    (getDocumentSymbolsFromWasm as jest.Mock).mockResolvedValue([
      {
        kind: "element",
        name: "App",
        detail: "system",
        range: { start: { line: 0, character: 0 }, end: { line: 2, character: 10 } },
        children: [],
      } as SrujaDocumentSymbol,
      {
        kind: "flow",
        name: "DoStuff",
        detail: "flow",
        range: { start: { line: 3, character: 2 }, end: { line: 5, character: 0 } },
        children: [],
      } as SrujaDocumentSymbol,
      {
        kind: "unknown",
        name: "Other",
        detail: "",
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } },
        children: [],
      } as unknown as SrujaDocumentSymbol,
    ]);

    const doc = {
      languageId: "sruja",
      getText: () => "system App {}",
      uri: { fsPath: "/test.sruja" },
    } as vscode.TextDocument;

    const provider = new SrujaDocumentSymbolProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDocumentSymbols(doc, new CancellationToken());
    expect(result).toHaveLength(3);
    expect(result![0].name).toBe("App");
    expect(result![0].kind).toBe(vscode.SymbolKind.Class);
    expect(result![1].kind).toBe(vscode.SymbolKind.Function);
    expect(result![2].kind).toBe(vscode.SymbolKind.Object);
  });

  it("maps all symbol kinds including requirement, adr, policy", async () => {
    const { getDocumentSymbolsFromWasm } = await import("./wasm");
    (getDocumentSymbolsFromWasm as jest.Mock).mockResolvedValue([
      { kind: "requirement", name: "R1", detail: "", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } }, children: [] } as unknown as import("./wasm").SrujaDocumentSymbol,
      { kind: "adr", name: "ADR-1", detail: "", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } }, children: [] } as unknown as import("./wasm").SrujaDocumentSymbol,
      { kind: "policy", name: "P1", detail: "", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } }, children: [] } as unknown as import("./wasm").SrujaDocumentSymbol,
      { kind: "view", name: "V1", detail: "", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } }, children: [] } as unknown as import("./wasm").SrujaDocumentSymbol,
      { kind: "scenario", name: "S1", detail: "", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } }, children: [] } as unknown as import("./wasm").SrujaDocumentSymbol,
    ]);
    const doc = {
      languageId: "sruja",
      getText: () => "system App {}",
      uri: { fsPath: "/test.sruja" },
    } as vscode.TextDocument;
    const provider = new SrujaDocumentSymbolProvider(asContext(new ExtensionContext()));
    const result = await provider.provideDocumentSymbols(doc, new CancellationToken());
    expect(result).toHaveLength(5);
    expect(result![0].kind).toBe(vscode.SymbolKind.Boolean);   // requirement
    expect(result![1].kind).toBe(vscode.SymbolKind.Enum);      // adr
    expect(result![2].kind).toBe(vscode.SymbolKind.Event);     // policy
    expect(result![3].kind).toBe(vscode.SymbolKind.Interface); // view
    expect(result![4].kind).toBe(vscode.SymbolKind.Method);    // scenario
  });
});
