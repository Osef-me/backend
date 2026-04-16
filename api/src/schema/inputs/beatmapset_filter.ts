import { builder } from "../builder.ts";
import { BoolFilterInput, StringFilterInput } from "./filters.ts";
import type { BoolFilter, StringFilter } from "./filters.ts";

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

export type BeatmapsetFilter = {
  artist?: StringFilter | null;
  title?: StringFilter | null;
  creator?: StringFilter | null;
  source?: StringFilter | null;
  isExplicit?: BoolFilter | null;
  isFeatured?: BoolFilter | null;
  hasVideo?: BoolFilter | null;
  hasStoryboard?: BoolFilter | null;
};
