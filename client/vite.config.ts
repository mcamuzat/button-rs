/// <reference types="vitest/config" />
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The dev site may be shared through an ngrok tunnel (any subdomain).
const NGROK_HOSTS = [".ngrok-free.dev", ".ngrok-free.app", ".ngrok.app", ".ngrok.io"];

export default defineConfig({
  // Relative paths: the build works from any folder.
  base: "./",
  plugins: [react(), tailwindcss()],
  server: { allowedHosts: NGROK_HOSTS },
  preview: { allowedHosts: NGROK_HOSTS },
  test: { environment: "node" },
});
