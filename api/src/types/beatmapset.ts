import type { BoolFilter, StringFilter } from "./filters.ts";

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
