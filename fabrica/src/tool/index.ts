import type { InferSchema, SchemaSource } from "../schema/index.ts";
import type { ToolDefinition } from "./types.ts";

export function defineTool<
  const SIn extends SchemaSource<unknown>,
  const SOut extends SchemaSource<unknown>,
>(def: ToolDefinition<SIn, SOut>): ToolDefinition<SIn, SOut> {
  return def;
}

type DD = ReturnType<typeof defineTool>;
export type ToolInput<D extends DD> = InferSchema<D["input"]>;
export type ToolOutput<D extends DD> = InferSchema<D["output"]>;
