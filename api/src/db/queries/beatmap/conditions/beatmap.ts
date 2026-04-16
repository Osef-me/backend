import type { SQL } from "drizzle-orm";
import { beatmap } from "@/db/schemas/beatmap.ts";
import { applyFloatFilter, applyIntFilter, applyStringFilter } from "@/db/queries/filters/index.ts";
import type { BeatmapFilter } from "@/types/beatmap.ts";

export function beatmapFieldConditions(f: BeatmapFilter | null | undefined): SQL[] {
  if (!f) return [];
  return [
    ...applyIntFilter(beatmap.id, f.id),
    ...applyIntFilter(beatmap.osuId, f.osuId),
    ...applyIntFilter(beatmap.mode, f.mode),
    ...applyStringFilter(beatmap.status, f.status),
    ...applyFloatFilter(beatmap.bpm, f.bpm),
    ...applyFloatFilter(beatmap.cs, f.cs),
    ...applyFloatFilter(beatmap.ar, f.ar),
    ...applyFloatFilter(beatmap.od, f.od),
    ...applyFloatFilter(beatmap.hp, f.hp),
    ...applyIntFilter(beatmap.totalTime, f.totalTime),
    ...applyIntFilter(beatmap.drainTime, f.drainTime),
    ...applyIntFilter(beatmap.maxCombo, f.maxCombo),
    ...applyIntFilter(beatmap.countCircles, f.countCircles),
    ...applyIntFilter(beatmap.countSliders, f.countSliders),
    ...applyIntFilter(beatmap.countSpinners, f.countSpinners),
  ];
}
