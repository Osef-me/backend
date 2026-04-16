import { BeatmapManiaSkillRef, BeatmapRatingRef } from "./refs.ts";

BeatmapRatingRef.implement({
  fields: (t) => ({
    id: t.exposeInt("id"),
    beatmapId: t.exposeInt("beatmapId"),
    ratingType: t.exposeString("ratingType"),
    rating: t.float({ resolve: (r) => parseFloat(r.rating) }),
    createdAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (r) => r.createdAt ?? null,
    }),
    updatedAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (r) => r.updatedAt ?? null,
    }),
    skill: t.field({
      type: BeatmapManiaSkillRef,
      nullable: true,
      resolve: async (rating, _, ctx) => {
        const result = await ctx.db.query.beatmapManiaSkill.findFirst({
          where: (s, { eq }) => eq(s.ratingId, rating.id),
        });
        return result ?? null;
      },
    }),
  }),
});
