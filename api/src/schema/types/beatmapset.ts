import { BeatmapRef, BeatmapsetRef } from "./refs.ts";

BeatmapsetRef.implement({
  fields: (t) => ({
    id: t.exposeInt("id"),
    osuId: t.exposeInt("osuId", { nullable: true }),
    artist: t.exposeString("artist"),
    artistUnicode: t.exposeString("artistUnicode", { nullable: true }),
    title: t.exposeString("title"),
    titleUnicode: t.exposeString("titleUnicode", { nullable: true }),
    creator: t.exposeString("creator"),
    source: t.exposeString("source", { nullable: true }),
    tags: t.field({
      type: ["String"],
      nullable: true,
      resolve: (bs) => bs.tags ?? null,
    }),
    hasVideo: t.exposeBoolean("hasVideo"),
    hasStoryboard: t.exposeBoolean("hasStoryboard"),
    isExplicit: t.exposeBoolean("isExplicit"),
    isFeatured: t.exposeBoolean("isFeatured"),
    coverUrl: t.exposeString("coverUrl", { nullable: true }),
    previewUrl: t.exposeString("previewUrl", { nullable: true }),
    osuFileUrl: t.exposeString("osuFileUrl", { nullable: true }),
    osuStatusChangedAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (bs) => bs.osuStatusChangedAt ?? null,
    }),
    createdAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (bs) => bs.createdAt ?? null,
    }),
    updatedAt: t.field({
      type: "DateTime",
      nullable: true,
      resolve: (bs) => bs.updatedAt ?? null,
    }),
    beatmaps: t.field({
      type: [BeatmapRef],
      resolve: (bs, _, ctx) =>
        ctx.db.query.beatmap.findMany({
          where: (b, { eq }) => eq(b.beatmapsetId, bs.id),
        }),
    }),
  }),
});
