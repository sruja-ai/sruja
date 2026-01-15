import { useEffect, useMemo, useRef } from "react";
import type * as monacoTypes from "monaco-editor";
import type { SrujaModelDump } from "@sruja/shared";
import { useSelectionStore } from "../stores/viewStore";
import { buildDslSelectionIndex } from "../utils/dslSelectionIndex";

interface DslSelectionSyncParams {
  editor: monacoTypes.editor.IStandaloneCodeEditor | null;
  monaco: typeof import("monaco-editor") | null;
  dslSource: string | null;
  model: SrujaModelDump | null;
}

export function useDslSelectionSync({
  editor,
  monaco,
  dslSource,
  model,
}: DslSelectionSyncParams) {
  const selectNode = useSelectionStore((s) => s.selectNode);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const selectionSource = useSelectionStore((s) => s.selectionSource);
  const decorationIdsRef = useRef<string[]>([]);

  const index = useMemo(
    () => buildDslSelectionIndex(dslSource || "", model),
    [dslSource, model]
  );

  useEffect(() => {
    if (!editor || !monaco) return;

    const disposable = editor.onDidChangeCursorPosition((event) => {
      const elementId = index.lineToElementId.get(event.position.lineNumber);
      if (elementId && elementId !== selectedNodeId) {
        selectNode(elementId, "code");
      }
    });

    return () => disposable.dispose();
  }, [editor, index, monaco, selectNode, selectedNodeId]);

  useEffect(() => {
    if (!editor || !monaco) return;

    if (!selectedNodeId) {
      if (decorationIdsRef.current.length > 0) {
        decorationIdsRef.current = editor.deltaDecorations(decorationIdsRef.current, []);
      }
      return;
    }

    const lineNumber = index.elementIdToLine.get(selectedNodeId);
    if (!lineNumber) {
      if (decorationIdsRef.current.length > 0) {
        decorationIdsRef.current = editor.deltaDecorations(decorationIdsRef.current, []);
      }
      return;
    }

    const range = new monaco.Range(lineNumber, 1, lineNumber, 1);
    decorationIdsRef.current = editor.deltaDecorations(decorationIdsRef.current, [
      {
        range,
        options: {
          isWholeLine: true,
          className: "dsl-selection-highlight",
        },
      },
    ]);

    if (selectionSource !== "code") {
      editor.revealLineInCenterIfOutsideViewport(lineNumber);
    }
  }, [editor, index, monaco, selectedNodeId, selectionSource]);
}
