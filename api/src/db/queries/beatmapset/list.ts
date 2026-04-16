import { count, desc } from "drizzle-orm";
import { beatmapset } from "@/db/schemas/beatmapset.ts";
import { buildWhere } from "@/db/queries/filters/index.ts";
import type { db as DB } from "@/db/client.ts";
import type { BeatmapsetFilter } from "@/types/beatmapset.ts";
import type { Pagination } from "@/types/pagination.ts";
import { beatmapsetConditions } from "./conditions.ts";

export async function listBeatmapsets(
  db: typeof DB,
  filter?: BeatmapsetFilter | null,
  pagination?: Pagination | null,
) {
  const limit = Math.min(pagination?.limit ?? 20, 100);
  const offset = pagination?.offset ?? 0;
  const where = buildWhere(beatmapsetConditions(filter));

  const [{ total }, items] = await Promise.all([
    db.select({ total: count() }).from(beatmapset).where(where).then(([r]) => r),
    db.select().from(beatmapset).where(where).orderBy(desc(beatmapset.createdAt)).limit(limit).offset(offset),
  ]);

  return { items, total: Number(total) };
}
