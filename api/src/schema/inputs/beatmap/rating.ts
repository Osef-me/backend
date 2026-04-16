import { builder } from "@/schema/builder.ts";
import { FloatFilterInput } from "@/schema/inputs/filters/index.ts";
import { ManiaSkillFilterInput } from "./skill.ts";

export const RatingFilterInput = builder.inputType("RatingFilter", {
  fields: (t) => ({
    type: t.string({ required: true }),
    value: t.field({ type: FloatFilterInput, required: false }),
    skill: t.field({ type: ManiaSkillFilterInput, required: false }),
  }),
});
