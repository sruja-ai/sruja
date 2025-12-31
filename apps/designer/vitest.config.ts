// apps/designer/vitest.config.ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html", "lcov"],
      exclude: [
        "node_modules/",
        "src/test/",
        "**/*.d.ts",
        "**/*.config.*",
        "**/dist/",
        "**/build/",
        "**/*.test.{ts,tsx}",
        "**/__tests__/**",
        "**/__mocks__/**",
        "**/index.ts",
        "**/index.tsx",
      ],
      include: ["src/**/*.{ts,tsx}"],
      thresholds: {
        // FAANG-level coverage thresholds
        lines: 90,
        functions: 90,
        branches: 85,
        statements: 90,
      },
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
