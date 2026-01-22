// @sruja/vscode-extension - ESLint configuration
import baseConfig from "@sruja/eslint-config";

export default [
  ...baseConfig,
  {
    ignores: ["dist/", "out/", "wasm/rust/sruja_wasm.js", "scripts/", "*.vsix", ".vscode-test/"],
  },
];
