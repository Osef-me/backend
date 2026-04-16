import { builder } from "../builder.ts";
import { BeatmapsetRef } from "../types/refs.ts";
import { BeatmapsetFilterInput } from "../inputs/beatmapset_filter.ts";
import { PaginationInput } from "../inputs/pagination.ts";
import { getBeatmapset, listBeatmapsets } from "../../db/queries/beatmapset.ts";

const BeatmapsetListResult = builder.objectRef<{
  items: (typeof BeatmapsetRef)[];
  total: number;
}>("BeatmapsetListResult");

BeatmapsetListResult.implement({
  fields: (t) => ({
    // deno-lint-ignore no-explicit-any
    items: t.field({ type: [BeatmapsetRef], resolve: (r: any) => r.items }),
    // deno-lint-ignore no-explicit-any
    total: t.int({ resolve: (r: any) => r.total }),
  }),
});

builder.queryFields((t) => ({
  beatmapset: t.field({
    type: BeatmapsetRef,
    nullable: true,
    args: {
      id: t.arg.int({ required: false }),
      osuId: t.arg.int({ required: false }),
    },
    resolve: (_, args, ctx) => getBeatmapset(ctx.db, args),
  }),

  beatmapsets: t.field({
    type: BeatmapsetListResult,
    args: {
      filter: t.arg({ type: BeatmapsetFilterInput, required: false }),
      pagination: t.arg({ type: PaginationInput, required: false }),
    },
    resolve: (_, args, ctx) =>
      listBeatmapsets(ctx.db, args.filter, args.pagination),
  }),
}));
