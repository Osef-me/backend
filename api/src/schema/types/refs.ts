// All object refs defined here to break circular dependencies between type files.
import { builder } from "../builder.ts";
import type { InferSelectModel } from "drizzle-orm";
import type { beatmapset } from "../../db/schemas/beatmapset.ts";
import type { beatmap } from "../../db/schemas/beatmap.ts";
import type { beatmapDuplicate } from "../../db/schemas/beatmap_duplicate.ts";
import type { beatmapRating } from "../../db/schemas/beatmap_rating.ts";
import type { beatmapManiaSkill } from "../../db/schemas/beatmap_mania_skill.ts";
import type { beatmapManiaRatio } from "../../db/schemas/beatmap_mania_ratio.ts";

export type BeatmapsetRow = InferSelectModel<typeof beatmapset>;
export type BeatmapRow = InferSelectModel<typeof beatmap>;
export type BeatmapDuplicateRow = InferSelectModel<typeof beatmapDuplicate>;
export type BeatmapRatingRow = InferSelectModel<typeof beatmapRating>;
export type BeatmapManiaSkillRow = InferSelectModel<typeof beatmapManiaSkill>;
export type BeatmapManiaRatioRow = InferSelectModel<typeof beatmapManiaRatio>;

export const BeatmapsetRef = builder.objectRef<BeatmapsetRow>("Beatmapset");
export const BeatmapRef = builder.objectRef<BeatmapRow>("Beatmap");
export const BeatmapDuplicateRef = builder.objectRef<BeatmapDuplicateRow>("BeatmapDuplicate");
export const BeatmapRatingRef = builder.objectRef<BeatmapRatingRow>("BeatmapRating");
export const BeatmapManiaSkillRef = builder.objectRef<BeatmapManiaSkillRow>("BeatmapManiaSkill");
export const BeatmapManiaRatioRef = builder.objectRef<BeatmapManiaRatioRow>("BeatmapManiaRatio");
