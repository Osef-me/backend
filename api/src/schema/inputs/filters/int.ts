import { builder } from "@/schema/builder.ts";

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
