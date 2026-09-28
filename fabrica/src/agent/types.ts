import type { SchemaSource } from "../schema/index.ts";

export interface AgentDefinition<
  SIn extends SchemaSource<unknown>,
  SOut extends SchemaSource<unknown>,
> {
  input: SIn;
  output: SOut;
}
