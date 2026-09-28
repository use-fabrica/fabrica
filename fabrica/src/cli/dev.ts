import { defineCommand } from "citty";

export default defineCommand({
  meta: { name: "dev", description: "Runs a fabrica project (stub)" },
  run: ({ args }) => {
    console.log(args);
  },
});
