import { builder } from "@/schema/builder.ts";
import { BeatmapsetRef } from "@/schema/types/refs.ts";
import { getBeatmapset } from "@/db/queries/beatmapset/index.ts";

builder.queryField("beatmapset", (t) =>
  t.field({
    type: BeatmapsetRef,
    nullable: true,
    args: {
      id: t.arg.int({ required: false }),
      osuId: t.arg.int({ required: false }),
    },
    resolve: (_, args, ctx) => getBeatmapset(ctx.db, args),
  }));
