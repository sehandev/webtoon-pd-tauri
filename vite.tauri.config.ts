import tailwindcss from "@tailwindcss/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import viteTsConfigPaths from "vite-tsconfig-paths";

const config = defineConfig({
  root: "src/tauri",
  publicDir: "../../public",
  plugins: [
    viteTsConfigPaths({
      projects: ["../../tsconfig.json"],
    }),
    tailwindcss(),
    viteReact(),
  ],
  build: {
    outDir: "../../dist/tauri",
    emptyOutDir: true,
  },
});

export default config;
