import { builder } from "@/schema/builder.ts";

export const StringFilterInput = builder.inputType("StringFilter", {
  fields: (t) => ({
    eq: t.string({ required: false }),
    contains: t.string({ required: false }),
    in: t.field({ type: ["String"], required: false }),
  }),
});
