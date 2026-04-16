import { builder } from "@/schema/builder.ts";
import { FloatFilterInput, IntFilterInput, StringFilterInput } from "@/schema/inputs/filters/index.ts";
import { BeatmapsetFilterInput } from "@/schema/inputs/beatmapset/index.ts";
import { ManiaRatioFilterInput } from "./ratio.ts";
import { RatingFilterInput } from "./rating.ts";

export const BeatmapFilterInput = builder.inputType("BeatmapFilter", {
  fields: (t) => ({
    id: t.field({ type: IntFilterInput, required: false }),
    osuId: t.field({ type: IntFilterInput, required: false }),
    mode: t.field({ type: IntFilterInput, required: false }),
    status: t.field({ type: StringFilterInput, required: false }),
    bpm: t.field({ type: FloatFilterInput, required: false }),
    cs: t.field({ type: FloatFilterInput, required: false }),
    ar: t.field({ type: FloatFilterInput, required: false }),
    od: t.field({ type: FloatFilterInput, required: false }),
    hp: t.field({ type: FloatFilterInput, required: false }),
    totalTime: t.field({ type: IntFilterInput, required: false }),
    drainTime: t.field({ type: IntFilterInput, required: false }),
    maxCombo: t.field({ type: IntFilterInput, required: false }),
    countCircles: t.field({ type: IntFilterInput, required: false }),
    countSliders: t.field({ type: IntFilterInput, required: false }),
    countSpinners: t.field({ type: IntFilterInput, required: false }),
    beatmapset: t.field({ type: BeatmapsetFilterInput, required: false }),
    ratio: t.field({ type: ManiaRatioFilterInput, required: false }),
    rating: t.field({ type: RatingFilterInput, required: false }),
  }),
});
