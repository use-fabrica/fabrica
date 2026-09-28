#!/usr/bin/env node

import { defineCommand, runMain } from "citty";

const main = defineCommand({
  meta: {
    name: "fabrica",
    description: "File-based TypeScript agents, tools, and durable workflows",
    version: "0.0.0",
  },
  args: {},
  subCommands: {
    init: () => import("./init.ts").then((m) => m.default),
    dev: () => import("./dev.ts").then((m) => m.default),
  },
});

runMain(main).catch((e) => {
  console.error(e);
  process.exit(1);
});
