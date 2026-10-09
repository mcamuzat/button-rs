/// <reference types="vitest/config" />
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  // Relative paths: the build works from any folder.
  base: "./",
  plugins: [react(), tailwindcss()],
  test: { environment: "node" },
});
