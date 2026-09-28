import type { StandardSchemaV1 } from "@standard-schema/spec";

export type JsonSchema = Record<string, unknown>;

export interface CustomSchema<T> {
  validate: (value: unknown) => StandardSchemaV1.Result<T> | Promise<StandardSchemaV1.Result<T>>;
  jsonSchema: () => JsonSchema;
}

export type SchemaSource<T> = StandardSchemaV1<unknown, T> | CustomSchema<T>;

export type InferSchema<S> =
  S extends StandardSchemaV1<any, infer O> ? O : S extends CustomSchema<infer O> ? O : never;
