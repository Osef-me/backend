import { count, type SQL } from "drizzle-orm";
import { beatmap } from "@/db/schemas/beatmap.ts";
import { applyJoins } from "./joins.ts";
import type { BeatmapFilter } from "@/types/beatmap.ts";
import type { db as DB } from "@/db/client.ts";

export async function countBeatmaps(
  db: typeof DB,
  filter: BeatmapFilter | null | undefined,
  where: SQL | undefined,
): Promise<number> {
  const base = db.select({ total: count() }).from(beatmap);
  const q = applyJoins(base, filter);
  const [{ total }] = await q.where(where);
  return Number(total);
}
