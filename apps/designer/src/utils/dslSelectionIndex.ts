// apps/designer/src/utils/dslSelectionIndex.ts
// Utility for mapping DSL lines to element IDs and vice versa.
import type { SrujaModelDump } from "@sruja/shared";

export interface DslSelectionIndex {
  elementIdToLine: Map<string, number>;
  lineToElementId: Map<number, string>;
}

type BraceCounts = {
  text: string;
  openBraces: number;
  closeBraces: number;
  hasContent: boolean;
};

const assignmentRegex = /^\s*([A-Za-z_][\w.-]*)\s*=\s*/;

function analyzeLine(line: string): BraceCounts {
  let text = "";
  let openBraces = 0;
  let closeBraces = 0;
  let inString = false;
  let escaped = false;

  for (let i = 0; i < line.length; i += 1) {
    const char = line[i];
    const nextChar = i + 1 < line.length ? line[i + 1] : "";

    if (!inString && char === "/" && nextChar === "/") {
      break;
    }

    if (!escaped && char === '"') {
      inString = !inString;
    }

    if (!inString) {
      if (char === "{") openBraces += 1;
      if (char === "}") closeBraces += 1;
    }

    text += char;
    escaped = char === "\\" && !escaped;
  }

  const hasContent = text.trim().length > 0;
  return { text, openBraces, closeBraces, hasContent };
}

function resolveElementId(
  localId: string,
  elementIds: Set<string>,
  elementStack: string[]
): string | null {
  if (elementIds.has(localId)) {
    return localId;
  }

  if (elementStack.length > 0) {
    const parentId = elementStack[elementStack.length - 1];
    const fullId = `${parentId}.${localId}`;
    if (elementIds.has(fullId)) {
      return fullId;
    }
  }

  return null;
}

export function buildDslSelectionIndex(
  dslSource: string,
  model: SrujaModelDump | null
): DslSelectionIndex {
  const elementIdToLine = new Map<string, number>();
  const lineToElementId = new Map<number, string>();

  if (!dslSource || !model?.elements) {
    return { elementIdToLine, lineToElementId };
  }

  const elementIds = new Set(Object.keys(model.elements));
  const elementStack: string[] = [];
  const blockStack: Array<"element" | "other"> = [];
  let pendingElementId: string | null = null;

  const lines = dslSource.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const lineNumber = index + 1;
    const { text, openBraces, closeBraces, hasContent } = analyzeLine(lines[index]);
    const trimmed = text.trim();

    if (pendingElementId && hasContent && trimmed !== "{" && !trimmed.startsWith("{")) {
      pendingElementId = null;
    }

    if (pendingElementId && (trimmed === "{" || trimmed.startsWith("{"))) {
      blockStack.push("element");
      elementStack.push(pendingElementId);
      pendingElementId = null;
    }

    const assignmentMatch = text.match(assignmentRegex);
    let elementIdForLine: string | null = null;

    if (assignmentMatch) {
      const localId = assignmentMatch[1];
      elementIdForLine = resolveElementId(localId, elementIds, elementStack);

      if (elementIdForLine && !elementIdToLine.has(elementIdForLine)) {
        elementIdToLine.set(elementIdForLine, lineNumber);
      }
    }

    if (elementIdForLine) {
      lineToElementId.set(lineNumber, elementIdForLine);
    } else if (hasContent && elementStack.length > 0) {
      lineToElementId.set(lineNumber, elementStack[elementStack.length - 1]);
    }

    let remainingOpenBraces = openBraces;
    if (elementIdForLine && remainingOpenBraces > 0) {
      blockStack.push("element");
      elementStack.push(elementIdForLine);
      remainingOpenBraces -= 1;
    } else if (elementIdForLine && remainingOpenBraces === 0) {
      pendingElementId = elementIdForLine;
    }

    while (remainingOpenBraces > 0) {
      blockStack.push("other");
      remainingOpenBraces -= 1;
    }

    let remainingCloseBraces = closeBraces;
    while (remainingCloseBraces > 0) {
      const closed = blockStack.pop();
      if (closed === "element") {
        elementStack.pop();
      }
      remainingCloseBraces -= 1;
    }
  }

  return { elementIdToLine, lineToElementId };
}
