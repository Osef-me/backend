// `any` isolated here — Drizzle's builder changes type with each join.
// All other files stay fully typed.
import { and, eq } from "drizzle-orm";
import { beatmap } from "@/db/schemas/beatmap.ts";
import { beatmapset } from "@/db/schemas/beatmapset.ts";
import { beatmapManiaRatio } from "@/db/schemas/beatmap_mania_ratio.ts";
import { beatmapRating } from "@/db/schemas/beatmap_rating.ts";
import { beatmapManiaSkill } from "@/db/schemas/beatmap_mania_skill.ts";
import type { BeatmapFilter } from "@/types/beatmap.ts";

// deno-lint-ignore no-explicit-any
type Q = any;

const joinBeatmapset = (q: Q) => q.leftJoin(beatmapset, eq(beatmap.beatmapsetId, beatmapset.id));
const joinRatio = (q: Q) => q.leftJoin(beatmapManiaRatio, eq(beatmapManiaRatio.beatmapId, beatmap.id));
const joinRating = (q: Q, type: string) => q.innerJoin(
  beatmapRating,
  and(eq(beatmapRating.beatmapId, beatmap.id), eq(beatmapRating.ratingType, type)),
);
const joinSkill = (q: Q) => q.leftJoin(beatmapManiaSkill, eq(beatmapManiaSkill.ratingId, beatmapRating.id));

export function applyJoins(q: Q, filter: BeatmapFilter | null | undefined): Q {
  let result = q;
  if (filter?.beatmapset) result = joinBeatmapset(result);
  if (filter?.ratio) result = joinRatio(result);
  if (filter?.rating) {
    result = joinRating(result, filter.rating.type);
    if (filter.rating.skill) result = joinSkill(result);
  }
  return result;
}
