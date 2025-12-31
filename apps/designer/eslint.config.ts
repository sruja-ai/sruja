import js from "@eslint/js";
import { configs as tsConfigs, config as tsConfig } from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tsConfig(
  js.configs.recommended,
  ...tsConfigs.recommended,
  reactRefresh.configs.vite,
  {
    ignores: [
      "dist",
      "postcss.config.cjs",
      "public/wasm/wasm_exec.js",
      "scripts/**/*",
      "eslint_designer_output.txt",
    ],
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
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "no-console": ["warn", { allow: ["warn", "error", "info", "debug"] }],
      "no-empty": "off",
      "prefer-const": "warn",
      "react-refresh/only-export-components": "off",
    },
  },
  {
    rules: {
      // Prevent window globals in production code (dev-only pattern)
      "no-restricted-syntax": [
        "error",
        {
          selector: "MemberExpression[object.name='window'][property.name=/^__.*__$/]",
          message:
            "Window globals with double underscores (__*__) are for dev-only use. Wrap in 'if (import.meta.env.DEV)' check.",
        },
      ],
    },
  },
  {
    files: ["**/tests/**/*.ts", "**/*.spec.ts", "**/*.spec.tsx"],
    rules: {
      "no-console": "off",
    },
  }
);
