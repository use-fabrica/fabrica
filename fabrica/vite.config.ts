import { defineConfig } from "vite-plus";

export default defineConfig({
  pack: {
    entry: {
      cli: "src/cli/index.ts",
      agent: "src/agent/index.ts",
      tool: "src/tool/index.ts",
      workflow: "src/workflow/index.ts",
      config: "src/config/index.ts",
    },
    dts: { generator: "tsgo" },
    exports: { exclude: ["cli"] },
    sourcemap: true,
  },
});
