// @sruja/vscode-extension - ESLint configuration
import baseConfig from "@sruja/eslint-config";

export default [
  ...baseConfig,
  {
    ignores: ["dist/", "out/", "wasm/wasm_exec.js", "scripts/", "*.vsix", ".vscode-test/"],
  },
];
