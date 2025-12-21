// packages/ui/src/components/sruja-language.ts
// Monaco Editor language definition for Sruja DSL syntax highlighting

import type * as monaco from "monaco-editor";

export function registerSrujaLanguage(monaco: typeof import("monaco-editor")) {
  // Register the language
  monaco.languages.register({ id: "sruja" });

  // Define tokenizer rules
  monaco.languages.setMonarchTokensProvider("sruja", {
    tokenizer: {
      root: [
        // Comments
        [/\/\/.*$/, "comment"],
        [/\/\*[\s\S]*?\*\//, "comment"],

        // Strings
        [/"([^"\\]|\\.)*$/, "string.invalid"], // non-terminated string
        [/"/, { token: "string.quote", bracket: "@open", next: "@string" }],

        // Numbers
        [/\d+\.\d+/, "number.float"],
        [/\d+/, "number"],

        // Keywords
        [
          /\b(workspace|model|views|system|component|container|import|relation|requirements|adrs|functional|nonfunctional|constraint|true|false)\b/,
          "keyword",
        ],

        // Operators
        [/->/, "operator"],
        [/[:=]/, "operator"],
        [/[{}]/, "delimiter.bracket"],
        [/[[\]]/, "delimiter.array"],
        [/[(),]/, "delimiter"],

        // Identifiers
        [/[a-zA-Z_][a-zA-Z0-9_]*/, "identifier"],

        // Whitespace
        [/\s+/, "white"],
      ],

      string: [
        [/[^\\"]+/, "string"],
        [/\\./, "string.escape.invalid"],
        [/"/, { token: "string.quote", bracket: "@close", next: "@pop" }],
      ],
    },
  } as monaco.languages.IMonarchLanguage);

  // Configure language features
  monaco.languages.setLanguageConfiguration("sruja", {
    comments: {
      lineComment: "//",
      blockComment: ["/*", "*/"],
    },
    brackets: [
      ["{", "}"],
      ["[", "]"],
      ["(", ")"],
    ],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
    ],
    surroundingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
    ],
    indentationRules: {
      increaseIndentPattern: /^.*\{[^}]*$/,
      decreaseIndentPattern: /^\s*\}/,
    },
  });
}
