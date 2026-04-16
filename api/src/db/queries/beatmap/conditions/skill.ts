import type { SQL } from "drizzle-orm";
import { beatmapRating } from "@/db/schemas/beatmap_rating.ts";
import { beatmapManiaSkill } from "@/db/schemas/beatmap_mania_skill.ts";
import { applyFloatFilter } from "@/db/queries/filters/index.ts";
import type { RatingFilter } from "@/types/beatmap.ts";

function ratingValueConditions(f: RatingFilter): SQL[] {
  return applyFloatFilter(beatmapRating.rating, f.value);
}

function skillConditions(f: RatingFilter): SQL[] {
  if (!f.skill) return [];
  return [
    ...applyFloatFilter(beatmapManiaSkill.stream, f.skill.stream),
    ...applyFloatFilter(beatmapManiaSkill.jumpstream, f.skill.jumpstream),
    ...applyFloatFilter(beatmapManiaSkill.handstream, f.skill.handstream),
    ...applyFloatFilter(beatmapManiaSkill.stamina, f.skill.stamina),
    ...applyFloatFilter(beatmapManiaSkill.jackspeed, f.skill.jackspeed),
    ...applyFloatFilter(beatmapManiaSkill.chordjack, f.skill.chordjack),
    ...applyFloatFilter(beatmapManiaSkill.technical, f.skill.technical),
  ];
}

export function ratingConditions(f: RatingFilter | null | undefined): SQL[] {
  if (!f) return [];
  return [...ratingValueConditions(f), ...skillConditions(f)];
}
