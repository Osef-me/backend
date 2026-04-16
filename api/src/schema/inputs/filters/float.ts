import { builder } from "@/schema/builder.ts";

export const FloatFilterInput = builder.inputType("FloatFilter", {
  fields: (t) => ({
    eq: t.float({ required: false }),
    gt: t.float({ required: false }),
    gte: t.float({ required: false }),
    lt: t.float({ required: false }),
    lte: t.float({ required: false }),
  }),
});
