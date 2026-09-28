import type { InferSchema, SchemaSource } from "../schema/index.ts";
import type { AgentDefinition } from "./types.ts";

export function defineAgent<
  const SIn extends SchemaSource<unknown>,
  const SOut extends SchemaSource<unknown>,
>(def: AgentDefinition<SIn, SOut>): AgentDefinition<SIn, SOut> {
  return def;
}

type DD = ReturnType<typeof defineAgent>;
export type AgentInput<D extends DD> = InferSchema<D["input"]>;
export type AgentOutput<D extends DD> = InferSchema<D["output"]>;
