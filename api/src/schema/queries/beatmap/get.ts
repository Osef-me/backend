import { builder } from "@/schema/builder.ts";
import { BeatmapRef } from "@/schema/types/refs.ts";
import { getBeatmap } from "@/db/queries/beatmap/index.ts";

builder.queryField("beatmap", (t) =>
  t.field({
    type: BeatmapRef,
    nullable: true,
    args: {
      id: t.arg.int({ required: false }),
      osuId: t.arg.int({ required: false }),
    },
    resolve: (_, args, ctx) => getBeatmap(ctx.db, args),
  }));
