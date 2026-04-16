import { type SQL } from "drizzle-orm";
import { beatmap } from "@/db/schemas/beatmap.ts";
import { applyJoins } from "./joins.ts";
import type { BeatmapFilter } from "@/types/beatmap.ts";
import type { db as DB } from "@/db/client.ts";

type BeatmapRow = typeof beatmap.$inferSelect;

export async function fetchBeatmaps(
  db: typeof DB,
  filter: BeatmapFilter | null | undefined,
  where: SQL | undefined,
  order: ReturnType<typeof import("drizzle-orm").asc>[],
  limit: number,
  offset: number,
): Promise<BeatmapRow[]> {
  const base = db.select({ beatmap }).from(beatmap);
  const q = applyJoins(base, filter);
  const rows = await q.where(where).orderBy(...order).limit(limit).offset(offset);
  return rows.map((r: { beatmap: BeatmapRow }) => r.beatmap);
}
