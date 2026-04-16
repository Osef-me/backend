import { builder } from "@/schema/builder.ts";
import { BeatmapsetFilterInput } from "@/schema/inputs/beatmapset/index.ts";
import { PaginationInput } from "@/schema/inputs/pagination/index.ts";
import { listBeatmapsets } from "@/db/queries/beatmapset/index.ts";
import { BeatmapsetListResult } from "./result.ts";

builder.queryField("beatmapsets", (t) =>
  t.field({
    type: BeatmapsetListResult,
    args: {
      filter: t.arg({ type: BeatmapsetFilterInput, required: false }),
      pagination: t.arg({ type: PaginationInput, required: false }),
    },
    resolve: (_, args, ctx) => listBeatmapsets(ctx.db, args.filter, args.pagination),
  }));
