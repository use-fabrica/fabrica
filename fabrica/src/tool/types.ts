import type { SchemaSource } from "../schema/index.ts";

export interface ToolDefinition<
  SIn extends SchemaSource<unknown>,
  SOut extends SchemaSource<unknown>,
> {
  input: SIn;
  output: SOut;
}
