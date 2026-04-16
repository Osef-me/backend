import {
  BeatmapDuplicateRef,
  BeatmapManiaRatioRef,
  BeatmapRatingRef,
  BeatmapRef,
  BeatmapsetRef,
} from "./refs.ts";

BeatmapRef.implement({
  fields: (t) => ({
    id: t.exposeInt("id"),
    osuId: t.exposeInt("osuId", { nullable: true }),
    beatmapsetId: t.exposeInt("beatmapsetId", { nullable: true }),
    difficulty: t.exposeString("difficulty"),
    osuHash: t.exposeString("osuHash"),
    notesHash: t.exposeString("notesHash"),
    countCircles: t.exposeInt("countCircles"),
    countSliders: t.exposeInt("countSliders"),
    countSpinners: t.exposeInt("countSpinners"),
    maxCombo: t.exposeInt("maxCombo"),
    mainPattern: t.field({ type: "JSON", resolve: (b) => b.mainPattern }),
    drainTime: t.exposeInt("drainTime"),
    totalTime: t.exposeInt("totalTime"),
    bpm: t.float({ resolve: (b) => parseFloat(b.bpm) }),
    cs: t.float({ resolve: (b) => parseFloat(b.cs) }),
    ar: t.float({ resolve: (b) => parseFloat(b.ar) }),
    od: t.float({ resolve: (b) => parseFloat(b.od) }),
    hp: t.float({ resolve: (b) => parseFloat(b.hp) }),
    mode: t.exposeInt("mode"),
    status: t.exposeString("status"),
    createdAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (b) => b.createdAt ?? null,
    }),
    updatedAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (b) => b.updatedAt ?? null,
    }),
    beatmapset: t.field({
      type: BeatmapsetRef,
      nullable: true,
      resolve: async (b, _, ctx) => {
        if (!b.beatmapsetId) return null;
        return ctx.db.query.beatmapset.findFirst({
          where: (bs, { eq }) => eq(bs.id, b.beatmapsetId!),
        }) ?? null;
      },
    }),
    ratings: t.field({
      type: [BeatmapRatingRef],
      resolve: (b, _, ctx) =>
        ctx.db.query.beatmapRating.findMany({
          where: (r, { eq }) => eq(r.beatmapId, b.id),
        }),
    }),
    ratio: t.field({
      type: BeatmapManiaRatioRef,
      nullable: true,
      resolve: async (b, _, ctx) => {
        return ctx.db.query.beatmapManiaRatio.findFirst({
          where: (r, { eq }) => eq(r.beatmapId, b.id),
        }) ?? null;
      },
    }),
    duplicates: t.field({
      type: [BeatmapDuplicateRef],
      resolve: (b, _, ctx) =>
        ctx.db.query.beatmapDuplicate.findMany({
          where: (d, { eq }) => eq(d.canonicalBeatmapId, b.id),
        }),
    }),
  }),
});
