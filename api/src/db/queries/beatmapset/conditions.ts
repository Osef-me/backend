import { beatmapset } from "@/db/schemas/beatmapset.ts";
import { applyBoolFilter, applyStringFilter } from "@/db/queries/filters/index.ts";
import type { BeatmapsetFilter } from "@/types/beatmapset.ts";
import type { SQL } from "drizzle-orm";

export function beatmapsetConditions(f: BeatmapsetFilter | null | undefined): SQL[] {
  if (!f) return [];
  return [
    ...applyStringFilter(beatmapset.artist, f.artist),
    ...applyStringFilter(beatmapset.title, f.title),
    ...applyStringFilter(beatmapset.creator, f.creator),
    ...applyStringFilter(beatmapset.source, f.source),
    ...applyBoolFilter(beatmapset.isExplicit, f.isExplicit),
    ...applyBoolFilter(beatmapset.isFeatured, f.isFeatured),
    ...applyBoolFilter(beatmapset.hasVideo, f.hasVideo),
    ...applyBoolFilter(beatmapset.hasStoryboard, f.hasStoryboard),
  ];
}
