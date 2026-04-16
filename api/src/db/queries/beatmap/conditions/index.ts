import { buildWhere } from "@/db/queries/filters/index.ts";
import type { BeatmapFilter } from "@/types/beatmap.ts";
import { beatmapFieldConditions } from "./beatmap.ts";
import { beatmapsetConditions } from "./beatmapset.ts";
import { ratioConditions } from "./ratio.ts";
import { ratingConditions } from "./skill.ts";

export function buildBeatmapWhere(f: BeatmapFilter | null | undefined) {
  return buildWhere([
    ...beatmapFieldConditions(f),
    ...beatmapsetConditions(f?.beatmapset),
    ...ratioConditions(f?.ratio),
    ...ratingConditions(f?.rating),
  ]);
}
