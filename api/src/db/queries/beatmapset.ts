import { count, desc, type SQL } from "drizzle-orm";
import type { db as DB } from "../client.ts";
import { beatmapset } from "../schemas/beatmapset.ts";
import {
  applyBoolFilter,
  applyStringFilter,
  buildWhere,
} from "./filter_helpers.ts";
import type { BeatmapsetFilter } from "../../schema/inputs/beatmapset_filter.ts";
import type { Pagination } from "../../schema/inputs/pagination.ts";

function buildConditions(filter?: BeatmapsetFilter | null): SQL[] {
  if (!filter) return [];
  return [
    ...applyStringFilter(beatmapset.artist, filter.artist),
    ...applyStringFilter(beatmapset.title, filter.title),
    ...applyStringFilter(beatmapset.creator, filter.creator),
    ...applyStringFilter(beatmapset.source, filter.source),
    ...applyBoolFilter(beatmapset.isExplicit, filter.isExplicit),
    ...applyBoolFilter(beatmapset.isFeatured, filter.isFeatured),
    ...applyBoolFilter(beatmapset.hasVideo, filter.hasVideo),
    ...applyBoolFilter(beatmapset.hasStoryboard, filter.hasStoryboard),
  ];
}

export async function listBeatmapsets(
  db: typeof DB,
  filter?: BeatmapsetFilter | null,
  pagination?: Pagination | null,
) {
  const limit = Math.min(pagination?.limit ?? 20, 100);
  const offset = pagination?.offset ?? 0;
  const where = buildWhere(buildConditions(filter));

  const [{ total }] = await db.select({ total: count() }).from(beatmapset).where(where);

  const items = await db
    .select()
    .from(beatmapset)
    .where(where)
    .orderBy(desc(beatmapset.createdAt))
    .limit(limit)
    .offset(offset);

  return { items, total: Number(total) };
}

export function getBeatmapset(
  db: typeof DB,
  opts: { id?: number | null; osuId?: number | null },
) {
  if (opts.id != null) {
    return db.query.beatmapset.findFirst({ where: (bs, { eq }) => eq(bs.id, opts.id!) });
  }
  if (opts.osuId != null) {
    return db.query.beatmapset.findFirst({ where: (bs, { eq }) => eq(bs.osuId, opts.osuId!) });
  }
  return null;
}
