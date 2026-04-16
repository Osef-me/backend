import type { SQL } from "drizzle-orm";
import { beatmapManiaRatio } from "@/db/schemas/beatmap_mania_ratio.ts";
import { applyFloatFilter } from "@/db/queries/filters/index.ts";
import type { ManiaRatioFilter } from "@/types/beatmap.ts";

export function ratioConditions(f: ManiaRatioFilter | null | undefined): SQL[] {
  if (!f) return [];
  return [
    ...applyFloatFilter(beatmapManiaRatio.stream, f.stream),
    ...applyFloatFilter(beatmapManiaRatio.jumpstream, f.jumpstream),
    ...applyFloatFilter(beatmapManiaRatio.handstream, f.handstream),
    ...applyFloatFilter(beatmapManiaRatio.stamina, f.stamina),
    ...applyFloatFilter(beatmapManiaRatio.jackspeed, f.jackspeed),
    ...applyFloatFilter(beatmapManiaRatio.chordjack, f.chordjack),
    ...applyFloatFilter(beatmapManiaRatio.technical, f.technical),
  ];
}
