import SchemaBuilder from "@pothos/core";
import { GraphQLScalarType } from "graphql";
import type { db } from "../db/client.ts";

interface Context {
  db: typeof db;
}

const JSONScalar = new GraphQLScalarType({
  name: "JSON",
  serialize: (v) => v,
  parseValue: (v) => v,
});

const DateTimeScalar = new GraphQLScalarType({
  name: "DateTime",
  serialize: (v) => (v instanceof Date ? v.toISOString() : String(v)),
  parseValue: (v) => new Date(String(v)),
});

export const builder = new SchemaBuilder<{
  Context: Context;
  Scalars: {
    JSON: { Input: unknown; Output: unknown };
    DateTime: { Input: Date; Output: Date };
  };
}>({});

builder.addScalarType("JSON", JSONScalar, {});
builder.addScalarType("DateTime", DateTimeScalar, {});
builder.queryType({});
