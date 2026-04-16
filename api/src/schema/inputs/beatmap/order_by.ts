import { builder } from "@/schema/builder.ts";
import { OrderDirEnum } from "@/schema/inputs/pagination/index.ts";

export const BeatmapOrderByInput = builder.inputType("BeatmapOrderBy", {
  fields: (t) => ({
    bpm: t.field({ type: OrderDirEnum, required: false }),
    od: t.field({ type: OrderDirEnum, required: false }),
    cs: t.field({ type: OrderDirEnum, required: false }),
    ar: t.field({ type: OrderDirEnum, required: false }),
    hp: t.field({ type: OrderDirEnum, required: false }),
    totalTime: t.field({ type: OrderDirEnum, required: false }),
    drainTime: t.field({ type: OrderDirEnum, required: false }),
    maxCombo: t.field({ type: OrderDirEnum, required: false }),
    createdAt: t.field({ type: OrderDirEnum, required: false }),
  }),
});
