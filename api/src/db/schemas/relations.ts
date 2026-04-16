import { relations } from "drizzle-orm";
import { beatmapset } from "./beatmapset.ts";
import { beatmap } from "./beatmap.ts";
import { beatmapDuplicate } from "./beatmap_duplicate.ts";
import { beatmapRating } from "./beatmap_rating.ts";
import { beatmapManiaSkill } from "./beatmap_mania_skill.ts";
import { beatmapManiaRatio } from "./beatmap_mania_ratio.ts";

export const beatmapsetRelations = relations(beatmapset, ({ many }) => ({
  beatmaps: many(beatmap),
}));

export const beatmapRelations = relations(beatmap, ({ one, many }) => ({
  beatmapset: one(beatmapset, {
    fields: [beatmap.beatmapsetId],
    references: [beatmapset.id],
  }),
  ratings: many(beatmapRating),
  ratio: one(beatmapManiaRatio, {
    fields: [beatmap.id],
    references: [beatmapManiaRatio.beatmapId],
  }),
  duplicates: many(beatmapDuplicate),
}));

export const beatmapDuplicateRelations = relations(beatmapDuplicate, ({ one }) => ({
  canonical: one(beatmap, {
    fields: [beatmapDuplicate.canonicalBeatmapId],
    references: [beatmap.id],
  }),
}));

export const beatmapRatingRelations = relations(beatmapRating, ({ one }) => ({
  beatmap: one(beatmap, {
    fields: [beatmapRating.beatmapId],
    references: [beatmap.id],
  }),
  skill: one(beatmapManiaSkill, {
    fields: [beatmapRating.id],
    references: [beatmapManiaSkill.ratingId],
  }),
}));

export const beatmapManiaSkillRelations = relations(beatmapManiaSkill, ({ one }) => ({
  rating: one(beatmapRating, {
    fields: [beatmapManiaSkill.ratingId],
    references: [beatmapRating.id],
  }),
}));

export const beatmapManiaRatioRelations = relations(beatmapManiaRatio, ({ one }) => ({
  beatmap: one(beatmap, {
    fields: [beatmapManiaRatio.beatmapId],
    references: [beatmap.id],
  }),
}));
