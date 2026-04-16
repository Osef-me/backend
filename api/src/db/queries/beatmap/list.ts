import type { db as DB } from "@/db/client.ts";
import type { BeatmapFilter, BeatmapOrderBy } from "@/types/beatmap.ts";
import type { Pagination } from "@/types/pagination.ts";
import { buildBeatmapWhere } from "./conditions/index.ts";
import { buildOrderBy } from "./order.ts";
import { countBeatmaps } from "./count.ts";
import { fetchBeatmaps } from "./fetch.ts";

const clampLimit = (limit?: number | null) => Math.min(limit ?? 20, 100);

export async function listBeatmaps(
  db: typeof DB,
  filter?: BeatmapFilter | null,
  pagination?: Pagination | null,
  orderBy?: BeatmapOrderBy | null,
) {
  const limit = clampLimit(pagination?.limit);
  const offset = pagination?.offset ?? 0;
  const where = buildBeatmapWhere(filter);
  const order = buildOrderBy(orderBy);

  const [total, items] = await Promise.all([
    countBeatmaps(db, filter, where),
    fetchBeatmaps(db, filter, where, order, limit, offset),
  ]);

  return { items, total };
}
