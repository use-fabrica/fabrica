import { defineCommand } from "citty";

export default defineCommand({
  meta: { name: "init", description: "Scaffold a fabrica project (stub)" },
  run: ({ args }) => {
    console.log(args);
  },
});
