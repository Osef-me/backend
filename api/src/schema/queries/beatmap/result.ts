import { builder } from "@/schema/builder.ts";
import { BeatmapRef } from "@/schema/types/refs.ts";
import type { BeatmapRow } from "@/schema/types/refs.ts";

export const BeatmapListResult = builder.objectRef<{
  items: BeatmapRow[];
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
