import { builder } from "../builder.ts";
import { BeatmapRef } from "../types/refs.ts";
import { BeatmapFilterInput, BeatmapOrderByInput } from "../inputs/beatmap_filter.ts";
import { PaginationInput } from "../inputs/pagination.ts";
import { getBeatmap, listBeatmaps } from "../../db/queries/beatmap.ts";

const BeatmapListResult = builder.objectRef<{
  items: (typeof BeatmapRef)[];
  total: number;
}>("BeatmapListResult");

BeatmapListResult.implement({
  fields: (t) => ({
    // deno-lint-ignore no-explicit-any
    items: t.field({ type: [BeatmapRef], resolve: (r: any) => r.items }),
    // deno-lint-ignore no-explicit-any
    total: t.int({ resolve: (r: any) => r.total }),
  }),
});

builder.queryFields((t) => ({
  beatmap: t.field({
    type: BeatmapRef,
    nullable: true,
    args: {
      id: t.arg.int({ required: false }),
      osuId: t.arg.int({ required: false }),
    },
    resolve: (_, args, ctx) => getBeatmap(ctx.db, args),
  }),

  beatmaps: t.field({
    type: BeatmapListResult,
    args: {
      filter: t.arg({ type: BeatmapFilterInput, required: false }),
      pagination: t.arg({ type: PaginationInput, required: false }),
      orderBy: t.arg({ type: BeatmapOrderByInput, required: false }),
    },
    resolve: (_, args, ctx) =>
      listBeatmaps(ctx.db, args.filter, args.pagination, args.orderBy),
  }),
}));
