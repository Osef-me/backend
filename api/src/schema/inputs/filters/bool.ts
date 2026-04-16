import { builder } from "@/schema/builder.ts";

export const BoolFilterInput = builder.inputType("BoolFilter", {
  fields: (t) => ({
    eq: t.boolean({ required: false }),
  }),
});
