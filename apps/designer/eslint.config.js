import js from "@eslint/js";
import { configs as tsConfigs, config as tsConfig } from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tsConfig(
  js.configs.recommended,
  ...tsConfigs.recommended,
  reactRefresh.configs.vite,
  {
    ignores: ["dist", "postcss.config.cjs"],
  },
  {
    plugins: {
      "react-hooks": reactHooks,
    },
    rules: {
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
    },
  },
  {
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "no-console": ["warn", { allow: ["warn", "error", "info", "debug"] }],
      "no-empty": "off",
      "prefer-const": "warn",
      "react-refresh/only-export-components": "off",
    },
  },
  {
    files: ["**/tests/**/*.ts", "**/*.spec.ts", "**/*.spec.tsx"],
    rules: {
      "no-console": "off",
    },
  }
);
