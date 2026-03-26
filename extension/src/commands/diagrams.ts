import * as vscode from "vscode";
import { getMermaidFromWasm, getElementsFromWasm, wasmRangeToVscodeRange, getSequenceDiagramFromWasm, getDocumentSymbolsFromWasm } from "../wasm";
import { escapeMermaidForScript, getDiagramPreviewHtml } from "../diagramPreview";
import { findElementById } from "../elementLookup";
import { pickActiveSrujaDoc } from "../utils";

let diagramPreviewPanel: vscode.WebviewPanel | undefined;
let currentDocUri: string | undefined;
let currentConfigJson: string | undefined;
let changeSubscription: vscode.Disposable | undefined;
let debounceTimer: ReturnType<typeof setTimeout> | undefined;

export function registerDiagramCommands(context: vscode.ExtensionContext, isTest: boolean) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.openDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open diagram preview.");
        return;
      }

      currentConfigJson = JSON.stringify({ viewLevel: 1 });
      currentDocUri = doc.uri.toString();
      
      const mermaid = await getMermaidFromWasm(context, doc.getText(), currentConfigJson);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }

      diagramPreviewPanel = createDiagramPanel(context, "Sruja – Diagram Preview");
      const escapedMermaid = escapeMermaidForScript(mermaid);
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapedMermaid, currentDocUri);
      
      setupLiveUpdateListener(context);
    }),

    vscode.commands.registerCommand("sruja.openFocusedDiagramPreviewAt", async (arg?: unknown) => {
      const parsed =
        typeof arg === "object" && arg !== null
          ? (arg as { docUri?: string; viewLevel?: unknown; targetId?: unknown })
          : undefined;

      const docUriRaw = typeof parsed?.docUri === "string" ? parsed.docUri : undefined;
      const viewLevelRaw = typeof parsed?.viewLevel === "number" ? parsed.viewLevel : undefined;
      const viewLevel = viewLevelRaw === 1 || viewLevelRaw === 2 || viewLevelRaw === 3 ? viewLevelRaw : 2;
      const targetId = viewLevel === 1 ? undefined : (typeof parsed?.targetId === "string" ? parsed.targetId : undefined);

      let doc: vscode.TextDocument | undefined;
      if (docUriRaw) {
        try {
          doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(docUriRaw));
        } catch {
          doc = undefined;
        }
      } else {
        doc = vscode.window.activeTextEditor?.document;
      }

      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open diagram preview.");
        return;
      }

      currentConfigJson = JSON.stringify({ viewLevel, targetId });
      currentDocUri = doc.uri.toString();

      const mermaid = await getMermaidFromWasm(context, doc.getText(), currentConfigJson);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }

      const titleSuffix = targetId ? ` – L${viewLevel}: ${targetId}` : ` – L${viewLevel}`;
      diagramPreviewPanel = createDiagramPanel(context, "Sruja – Diagram Preview" + titleSuffix);
      const escapedMermaid = escapeMermaidForScript(mermaid);
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapedMermaid, currentDocUri);
      
      setupLiveUpdateListener(context);
    }),

    vscode.commands.registerCommand("sruja.openFocusedDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open diagram preview.");
        return;
      }

      const levelPick = await vscode.window.showQuickPick(
        [
          { label: "L1 (Context)", level: 1 },
          { label: "L2 (Container)", level: 2 },
          { label: "L3 (Component)", level: 3 },
        ],
        { placeHolder: "Choose diagram level" }
      );
      if (!levelPick) return;

      let targetId: string | undefined;
      if (levelPick.level !== 1) {
        const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath, doc.uri.toString(), doc.version);
        if (!elements?.length) {
          vscode.window.showWarningMessage(
            "Could not list elements for focus selection. Ensure the file parses and Sruja WASM is available."
          );
          return;
        }

        const items = elements
          .map((e) => ({
            label: e.id,
            description: e.kind,
            detail: e.title ?? undefined,
            id: e.id,
          }))
          .sort((a, b) => a.label.localeCompare(b.label));

        const picked = await vscode.window.showQuickPick(items, {
          placeHolder: "Choose a focus element ID",
          matchOnDescription: true,
          matchOnDetail: true,
        });
        if (!picked) return;
        targetId = picked.id;
      }

      currentConfigJson = JSON.stringify({ viewLevel: levelPick.level, targetId });
      currentDocUri = doc.uri.toString();

      const mermaid = await getMermaidFromWasm(context, doc.getText(), currentConfigJson);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }

      const titleSuffix = targetId ? ` – ${levelPick.label}: ${targetId}` : ` – ${levelPick.label}`;
      diagramPreviewPanel = createDiagramPanel(context, "Sruja – Diagram Preview" + titleSuffix);
      const escapedMermaid = escapeMermaidForScript(mermaid);
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapedMermaid, currentDocUri);
      
      setupLiveUpdateListener(context);
    }),

    vscode.commands.registerCommand("sruja.openSequenceDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open sequence diagram preview.");
        return;
      }

      const symbols = await getDocumentSymbolsFromWasm(context, doc.getText(), doc.uri.fsPath, doc.uri.toString(), doc.version);
      if (!symbols) {
        vscode.window.showWarningMessage(
          "Could not list scenarios/flows. Ensure the file parses and Sruja WASM is available."
        );
        return;
      }

      type ScenarioFlowPick = vscode.QuickPickItem & { seqKind: "scenario" | "flow"; id: string };

      const items: ScenarioFlowPick[] = symbols
        .filter((s) => s.kind === "scenario" || s.kind === "flow")
        .map((s) => ({
          label: s.name,
          description: s.kind === "flow" ? "Flow" : "Scenario",
          seqKind: s.kind === "flow" ? ("flow" as const) : ("scenario" as const),
          id: s.name,
        }))
        .sort((a, b) => a.label.localeCompare(b.label));

      if (items.length === 0) {
        vscode.window.showInformationMessage("No scenarios or flows found in this file.");
        return;
      }

      const picked = isTest
        ? items[0]
        : await vscode.window.showQuickPick<ScenarioFlowPick>(items, {
            placeHolder: "Choose a scenario or flow to render as a sequence diagram",
            matchOnDescription: true,
          });
      if (!picked) return;

      await vscode.commands.executeCommand("sruja.openSequenceDiagramPreviewAt", {
        docUri: doc.uri.toString(),
        kind: picked.seqKind,
        id: picked.id,
      });
    }),

    vscode.commands.registerCommand("sruja.openSequenceDiagramPreviewAt", async (arg?: unknown) => {
      const parsed =
        typeof arg === "object" && arg !== null
          ? (arg as { docUri?: string; kind?: "scenario" | "flow"; id?: string })
          : undefined;

      const docUriRaw = parsed?.docUri;
      const kind = parsed?.kind || "scenario";
      const id = parsed?.id;

      let doc: vscode.TextDocument | undefined;
      if (docUriRaw) {
        try {
          doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(docUriRaw));
        } catch {
          doc = undefined;
        }
      } else {
        doc = vscode.window.activeTextEditor?.document;
      }

      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open sequence diagram preview.");
        return;
      }
      if (!id || !id.trim()) {
        vscode.window.showWarningMessage("No scenario/flow ID provided.");
        return;
      }

      currentConfigJson = JSON.stringify({ kind, id });
      currentDocUri = doc.uri.toString();

      const mermaid = await getSequenceDiagramFromWasm(context, doc.getText(), currentConfigJson);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          `Could not render sequence diagram for ${kind} "${id}". Ensure the file parses and Sruja WASM is available.`
        );
        return;
      }

      diagramPreviewPanel = createDiagramPanel(context, `Sruja – Sequence Diagram – ${kind}: ${id}`);
      const escapedMermaid = escapeMermaidForScript(mermaid);
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapedMermaid, currentDocUri);
      
      setupLiveUpdateListener(context);
    })
  );
}

function setupLiveUpdateListener(context: vscode.ExtensionContext) {
  if (changeSubscription) {
    changeSubscription.dispose();
  }
  changeSubscription = vscode.workspace.onDidChangeTextDocument(async (e) => {
    if (e.document.uri.toString() === currentDocUri && diagramPreviewPanel) {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(async () => {
        if (!diagramPreviewPanel || !currentConfigJson) return;

        try {
          let mermaid: string | null;
          const config = JSON.parse(currentConfigJson);
          if (config.kind && config.id) {
            mermaid = await getSequenceDiagramFromWasm(
              context,
              e.document.getText(),
              currentConfigJson!
            );
          } else {
            mermaid = await getMermaidFromWasm(context, e.document.getText(), currentConfigJson!);
          }

          if (mermaid && diagramPreviewPanel) {
            diagramPreviewPanel.webview.postMessage({ command: "update", code: mermaid });
          }
        } catch (err) {
          console.error("Failed to update diagram:", err);
        }
      }, 500);
    }
  });
}

function createDiagramPanel(context: vscode.ExtensionContext, title: string): vscode.WebviewPanel {
  if (diagramPreviewPanel) {
    diagramPreviewPanel.dispose();
  }
  diagramPreviewPanel = vscode.window.createWebviewPanel(
    "srujaDiagramPreview",
    title,
    vscode.ViewColumn.Beside,
    { enableScripts: true }
  );
  diagramPreviewPanel.onDidDispose(() => {
    diagramPreviewPanel = undefined;
    currentDocUri = undefined;
    currentConfigJson = undefined;
    if (changeSubscription) {
      changeSubscription.dispose();
      changeSubscription = undefined;
    }
  });
  diagramPreviewPanel.webview.onDidReceiveMessage(
    async (message) => {
      if (message.command === 'jumpToElement' && message.elementId) {
        await jumpToElementInDocument(context, message.elementId, message.sourceUri);
      }
    },
    undefined,
    context.subscriptions
  );
  return diagramPreviewPanel;
}

async function jumpToElementInDocument(context: vscode.ExtensionContext, elementId: string, sourceUri?: string) {
  let doc: vscode.TextDocument | undefined;
  if (sourceUri) {
    try {
      doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(sourceUri));
    } catch {
      doc = pickActiveSrujaDoc();
    }
  } else {
    doc = pickActiveSrujaDoc();
  }
  
  if (!doc) return;
  const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath, doc.uri.toString(), doc.version);
  if (!elements) return;
  const element = findElementById(elements, elementId);
  if (element) {
    const editor = await vscode.window.showTextDocument(doc, { viewColumn: vscode.ViewColumn.One });
    const range = wasmRangeToVscodeRange(element.range);
    editor.selection = new vscode.Selection(range.start, range.start);
    editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
  }
}
