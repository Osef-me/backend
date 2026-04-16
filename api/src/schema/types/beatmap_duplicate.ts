import { BeatmapDuplicateRef, BeatmapRef } from "./refs.ts";

BeatmapDuplicateRef.implement({
  fields: (t) => ({
    id: t.exposeInt("id"),
    canonicalBeatmapId: t.exposeInt("canonicalBeatmapId"),
    osuId: t.exposeInt("osuId", { nullable: true }),
    osuHash: t.exposeString("osuHash"),
    notesHash: t.exposeString("notesHash"),
    createdAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (d) => d.createdAt ?? null,
    }),
    canonical: t.field({
      type: BeatmapRef,
      resolve: (d, _, ctx) =>
        ctx.db.query.beatmap.findFirst({
          where: (b, { eq }) => eq(b.id, d.canonicalBeatmapId),
        }),
    }),
  }),
});
