import { builder } from "@/schema/builder.ts";
import { BeatmapFilterInput, BeatmapOrderByInput } from "@/schema/inputs/beatmap/index.ts";
import { PaginationInput } from "@/schema/inputs/pagination/index.ts";
import { listBeatmaps } from "@/db/queries/beatmap/index.ts";
import { BeatmapListResult } from "./result.ts";

builder.queryField("beatmaps", (t) =>
  t.field({
    type: BeatmapListResult,
    args: {
      filter: t.arg({ type: BeatmapFilterInput, required: false }),
      pagination: t.arg({ type: PaginationInput, required: false }),
      orderBy: t.arg({ type: BeatmapOrderByInput, required: false }),
    },
    resolve: (_, args, ctx) => listBeatmaps(ctx.db, args.filter, args.pagination, args.orderBy),
  }));
