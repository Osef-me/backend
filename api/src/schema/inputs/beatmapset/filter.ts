import { builder } from "@/schema/builder.ts";
import { BoolFilterInput, StringFilterInput } from "@/schema/inputs/filters/index.ts";

export const BeatmapsetFilterInput = builder.inputType("BeatmapsetFilter", {
  fields: (t) => ({
    artist: t.field({ type: StringFilterInput, required: false }),
    title: t.field({ type: StringFilterInput, required: false }),
    creator: t.field({ type: StringFilterInput, required: false }),
    source: t.field({ type: StringFilterInput, required: false }),
    isExplicit: t.field({ type: BoolFilterInput, required: false }),
    isFeatured: t.field({ type: BoolFilterInput, required: false }),
    hasVideo: t.field({ type: BoolFilterInput, required: false }),
    hasStoryboard: t.field({ type: BoolFilterInput, required: false }),
  }),
});
