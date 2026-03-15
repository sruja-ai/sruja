import * as vscode from "vscode";
import { parseLintStderr, parseLintJson, parseLintOutput } from "./lintParser";
import { getDiagramPreviewHtml, escapeMermaidForScript } from "./diagramPreview";

describe("parseLintStderr", () => {
  it("returns empty array for empty stderr", () => {
    expect(parseLintStderr("", "file:///a.sruja")).toEqual([]);
  });

  it("parses location line and creates diagnostic", () => {
    const stderr = "[E001] Error: Missing description\n  --> file.sruja:2:3";
    const diags = parseLintStderr(stderr, "file:///file.sruja");
    expect(diags).toHaveLength(1);
    expect(diags[0].message).toBe("Missing description");
    expect(diags[0].code).toBe("E001");
    expect(diags[0].severity).toBe(vscode.DiagnosticSeverity.Error);
    expect(diags[0].range.start.line).toBe(1);
    expect(diags[0].range.start.character).toBe(2);
  });

  it("parses Warning severity", () => {
    const stderr = "[W001] Warning: Optional suggestion\n  --> file.sruja:1:0";
    const diags = parseLintStderr(stderr, "file:///file.sruja");
    expect(diags).toHaveLength(1);
    expect(diags[0].severity).toBe(vscode.DiagnosticSeverity.Warning);
  });

  it("parses Info severity", () => {
    const stderr = "[I001] Info: Hint\n  --> file.sruja:1:0";
    const diags = parseLintStderr(stderr, "file:///file.sruja");
    expect(diags).toHaveLength(1);
    expect(diags[0].severity).toBe(vscode.DiagnosticSeverity.Information);
  });

  it("ignores lines that do not match location pattern", () => {
    const stderr = "some random output\n  --> file.sruja:1:0";
    const diags = parseLintStderr(stderr, "file:///file.sruja");
    expect(diags).toHaveLength(1);
    expect(diags[0].message).toBe("Validation error");
  });
});

describe("parseLintJson", () => {
  it("returns null for invalid JSON", () => {
    expect(parseLintJson("not json", "file:///a.sruja")).toBeNull();
  });

  it("returns null when diagnostics is missing", () => {
    expect(parseLintJson('{"ok": true}', "file:///a.sruja")).toBeNull();
  });

  it("parses valid JSON diagnostics", () => {
    const stdout = JSON.stringify({
      ok: false,
      error_count: 1,
      warning_count: 0,
      diagnostics: [
        {
          code: "E001",
          severity: "error",
          message: "Missing description",
          location: { file: "a.sruja", line: 2, column: 5 },
        },
      ],
    });
    const diags = parseLintJson(stdout, "file:///a.sruja");
    expect(diags).toHaveLength(1);
    expect(diags![0].message).toBe("Missing description");
    expect(diags![0].code).toBe("E001");
    expect(diags![0].severity).toBe(vscode.DiagnosticSeverity.Error);
    expect(diags![0].range.start.line).toBe(1);
    expect(diags![0].range.start.character).toBe(4);
  });

  it("maps warning and info severity", () => {
    const stdout = JSON.stringify({
      ok: false,
      error_count: 0,
      warning_count: 1,
      diagnostics: [
        { code: "W1", severity: "warning", message: "Warn" },
        { code: "I1", severity: "info", message: "Info" },
      ],
    });
    const diags = parseLintJson(stdout, "file:///a.sruja");
    expect(diags).toHaveLength(2);
    expect(diags![0].severity).toBe(vscode.DiagnosticSeverity.Warning);
    expect(diags![1].severity).toBe(vscode.DiagnosticSeverity.Information);
  });

  it("uses line/column 0 when location is missing", () => {
    const stdout = JSON.stringify({
      ok: false,
      diagnostics: [{ code: "E1", severity: "error", message: "No loc" }],
    });
    const diags = parseLintJson(stdout, "file:///a.sruja");
    expect(diags).toHaveLength(1);
    expect(diags![0].range.start.line).toBe(0);
    expect(diags![0].range.start.character).toBe(0);
  });

  it("omits code when diagnostic has no code", () => {
    const stdout = JSON.stringify({
      ok: false,
      diagnostics: [{ severity: "error", message: "No code", location: { file: "a.sruja", line: 1, column: 0 } }],
    });
    const diags = parseLintJson(stdout, "file:///a.sruja");
    expect(diags).toHaveLength(1);
    expect(diags![0].code).toBeUndefined();
  });

  it("maps severity error to DiagnosticSeverity.Error", () => {
    const stdout = JSON.stringify({
      ok: false,
      diagnostics: [{ code: "E1", severity: "error", message: "Error msg", location: { file: "a.sruja", line: 2, column: 1 } }],
    });
    const diags = parseLintJson(stdout, "file:///a.sruja");
    expect(diags).toHaveLength(1);
    expect(diags![0].severity).toBe(vscode.DiagnosticSeverity.Error);
    expect(diags![0].range.start.line).toBe(1);
    expect(diags![0].range.start.character).toBe(0);
  });
});

describe("getDiagramPreviewHtml", () => {
  it("embeds mermaid code in script and div", () => {
    const html = getDiagramPreviewHtml("graph TD A-->B");
    expect(html).toContain("<div id=\"diagram\" class=\"mermaid\">");
    expect(html).toContain("graph TD A-->B");
    expect(html).toContain("mermaid.run");
  });

  it("embeds mermaid safely when input has no backticks", () => {
    const html = getDiagramPreviewHtml("graph LR A-->B");
    expect(html).toContain("graph LR A-->B");
  });
});

describe("parseLintOutput", () => {
  it("prefers JSON when valid", () => {
    const stdout = JSON.stringify({
      ok: false,
      diagnostics: [{ code: "E1", severity: "error", message: "From JSON" }],
    });
    const diags = parseLintOutput(stdout, "[E2] Error: From stderr\n  --> f:1:0", "file:///f");
    expect(diags).toHaveLength(1);
    expect(diags[0].message).toBe("From JSON");
  });
  it("falls back to stderr when JSON invalid", () => {
    const diags = parseLintOutput("not json", "[E1] Error: From stderr\n  --> f:1:0", "file:///f");
    expect(diags).toHaveLength(1);
    expect(diags[0].message).toBe("From stderr");
  });
});

describe("escapeMermaidForScript", () => {
  it("escapes backticks and backslash", () => {
    expect(escapeMermaidForScript("`code`")).toContain("\\`");
    expect(escapeMermaidForScript("a\\b")).toContain("\\\\");
  });
});
