import { builder } from "@/schema/builder.ts";
import { FloatFilterInput } from "@/schema/inputs/filters/index.ts";

export const ManiaSkillFilterInput = builder.inputType("ManiaSkillFilter", {
  fields: (t) => ({
    stream: t.field({ type: FloatFilterInput, required: false }),
    jumpstream: t.field({ type: FloatFilterInput, required: false }),
    handstream: t.field({ type: FloatFilterInput, required: false }),
    stamina: t.field({ type: FloatFilterInput, required: false }),
    jackspeed: t.field({ type: FloatFilterInput, required: false }),
    chordjack: t.field({ type: FloatFilterInput, required: false }),
    technical: t.field({ type: FloatFilterInput, required: false }),
  }),
});
