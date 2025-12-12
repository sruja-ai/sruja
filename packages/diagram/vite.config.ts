import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";

export default defineConfig({
  plugins: [react()],
  root: "test-app",
  server: {
    port: 3000,
    fs: {
      strict: false,
      allow: [".."],
    },
  },
  resolve: {
    alias: {},
  },
});
