import { builder } from "../builder.ts";

export const IntFilterInput = builder.inputType("IntFilter", {
  fields: (t) => ({
    eq: t.int({ required: false }),
    gt: t.int({ required: false }),
    gte: t.int({ required: false }),
    lt: t.int({ required: false }),
    lte: t.int({ required: false }),
    in: t.field({ type: ["Int"], required: false }),
  }),
});

export const FloatFilterInput = builder.inputType("FloatFilter", {
  fields: (t) => ({
    eq: t.float({ required: false }),
    gt: t.float({ required: false }),
    gte: t.float({ required: false }),
    lt: t.float({ required: false }),
    lte: t.float({ required: false }),
  }),
});

export const StringFilterInput = builder.inputType("StringFilter", {
  fields: (t) => ({
    eq: t.string({ required: false }),
    contains: t.string({ required: false }), // ILIKE with pg_trgm
    in: t.field({ type: ["String"], required: false }),
  }),
});

export const BoolFilterInput = builder.inputType("BoolFilter", {
  fields: (t) => ({
    eq: t.boolean({ required: false }),
  }),
});

// TypeScript types mirroring the Pothos inputs — used in DB query layer
export type IntFilter = {
  eq?: number | null;
  gt?: number | null;
  gte?: number | null;
  lt?: number | null;
  lte?: number | null;
  in?: number[] | null;
};

export type FloatFilter = {
  eq?: number | null;
  gt?: number | null;
  gte?: number | null;
  lt?: number | null;
  lte?: number | null;
};

export type StringFilter = {
  eq?: string | null;
  contains?: string | null;
  in?: string[] | null;
};

export type BoolFilter = {
  eq?: boolean | null;
};
