import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      // Direct our shared components and utils
      "@/": resolve(__dirname, "../src/"),
    },
  },
  build: {
    outDir: resolve(__dirname, "../dist-extension-gemini"),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        popup: resolve(__dirname, "popup.html"),
        sidepanel: resolve(__dirname, "sidepanel.html"),
        options: resolve(__dirname, "options.html"),
      },
    },
  },
});
