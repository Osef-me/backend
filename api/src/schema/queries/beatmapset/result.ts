import { builder } from "@/schema/builder.ts";
import { BeatmapsetRef } from "@/schema/types/refs.ts";
import type { BeatmapsetRow } from "@/schema/types/refs.ts";

export const BeatmapsetListResult = builder.objectRef<{
  items: BeatmapsetRow[];
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
