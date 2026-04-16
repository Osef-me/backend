import { and, asc, count, desc, eq, type SQL } from "drizzle-orm";
import type { db as DB } from "../client.ts";
import { beatmap } from "../schemas/beatmap.ts";
import { beatmapset } from "../schemas/beatmapset.ts";
import { beatmapManiaRatio } from "../schemas/beatmap_mania_ratio.ts";
import { beatmapRating } from "../schemas/beatmap_rating.ts";
import { beatmapManiaSkill } from "../schemas/beatmap_mania_skill.ts";
import {
  applyBoolFilter,
  applyFloatFilter,
  applyIntFilter,
  applyStringFilter,
  buildWhere,
} from "./filter_helpers.ts";
import type { BeatmapFilter, BeatmapOrderBy } from "../../schema/inputs/beatmap_filter.ts";
import type { Pagination } from "../../schema/inputs/pagination.ts";

function buildBeatmapConditions(filter?: BeatmapFilter | null): SQL[] {
  if (!filter) return [];
  return [
    ...applyIntFilter(beatmap.id, filter.id),
    ...applyIntFilter(beatmap.osuId, filter.osuId),
    ...applyIntFilter(beatmap.mode, filter.mode),
    ...applyStringFilter(beatmap.status, filter.status),
    ...applyFloatFilter(beatmap.bpm, filter.bpm),
    ...applyFloatFilter(beatmap.cs, filter.cs),
    ...applyFloatFilter(beatmap.ar, filter.ar),
    ...applyFloatFilter(beatmap.od, filter.od),
    ...applyFloatFilter(beatmap.hp, filter.hp),
    ...applyIntFilter(beatmap.totalTime, filter.totalTime),
    ...applyIntFilter(beatmap.drainTime, filter.drainTime),
    ...applyIntFilter(beatmap.maxCombo, filter.maxCombo),
    ...applyIntFilter(beatmap.countCircles, filter.countCircles),
    ...applyIntFilter(beatmap.countSliders, filter.countSliders),
    ...applyIntFilter(beatmap.countSpinners, filter.countSpinners),
  ];
}

function buildBeatmapsetConditions(filter?: BeatmapFilter["beatmapset"]): SQL[] {
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

function buildRatioConditions(filter?: BeatmapFilter["ratio"]): SQL[] {
  if (!filter) return [];
  return [
    ...applyFloatFilter(beatmapManiaRatio.stream, filter.stream),
    ...applyFloatFilter(beatmapManiaRatio.jumpstream, filter.jumpstream),
    ...applyFloatFilter(beatmapManiaRatio.handstream, filter.handstream),
    ...applyFloatFilter(beatmapManiaRatio.stamina, filter.stamina),
    ...applyFloatFilter(beatmapManiaRatio.jackspeed, filter.jackspeed),
    ...applyFloatFilter(beatmapManiaRatio.chordjack, filter.chordjack),
    ...applyFloatFilter(beatmapManiaRatio.technical, filter.technical),
  ];
}

function buildSkillConditions(filter?: BeatmapFilter["rating"]): SQL[] {
  if (!filter) return [];
  const conditions: SQL[] = [
    ...applyFloatFilter(beatmapRating.rating, filter.value),
  ];
  if (filter.skill) {
    conditions.push(
      ...applyFloatFilter(beatmapManiaSkill.stream, filter.skill.stream),
      ...applyFloatFilter(beatmapManiaSkill.jumpstream, filter.skill.jumpstream),
      ...applyFloatFilter(beatmapManiaSkill.handstream, filter.skill.handstream),
      ...applyFloatFilter(beatmapManiaSkill.stamina, filter.skill.stamina),
      ...applyFloatFilter(beatmapManiaSkill.jackspeed, filter.skill.jackspeed),
      ...applyFloatFilter(beatmapManiaSkill.chordjack, filter.skill.chordjack),
      ...applyFloatFilter(beatmapManiaSkill.technical, filter.skill.technical),
    );
  }
  return conditions;
}

const ORDERABLE = {
  bpm: beatmap.bpm,
  od: beatmap.od,
  cs: beatmap.cs,
  ar: beatmap.ar,
  hp: beatmap.hp,
  totalTime: beatmap.totalTime,
  drainTime: beatmap.drainTime,
  maxCombo: beatmap.maxCombo,
  createdAt: beatmap.createdAt,
} as const;

function buildOrderBy(orderBy?: BeatmapOrderBy | null) {
  if (!orderBy) return [desc(beatmap.createdAt)];
  return Object.entries(orderBy)
    .filter(([, dir]) => dir != null)
    .map(([field, dir]) => {
      const col = ORDERABLE[field as keyof typeof ORDERABLE];
      return dir === "asc" ? asc(col) : desc(col);
    });
}

export async function listBeatmaps(
  db: typeof DB,
  filter?: BeatmapFilter | null,
  pagination?: Pagination | null,
  orderBy?: BeatmapOrderBy | null,
) {
  const limit = Math.min(pagination?.limit ?? 20, 100);
  const offset = pagination?.offset ?? 0;

  const needsBeatmapset = !!filter?.beatmapset;
  const needsRatio = !!filter?.ratio;
  const needsRating = !!filter?.rating;
  const needsSkill = !!(filter?.rating?.skill);

  const conditions: SQL[] = [
    ...buildBeatmapConditions(filter),
    ...(needsBeatmapset ? buildBeatmapsetConditions(filter?.beatmapset) : []),
    ...(needsRatio ? buildRatioConditions(filter?.ratio) : []),
    ...(needsRating ? buildSkillConditions(filter?.rating) : []),
  ];

  const where = buildWhere(conditions);
  const order = buildOrderBy(orderBy);

  // Build base — always select beatmap columns
  // Joins are applied conditionally to avoid row duplication
  let q = db.select({ beatmap }).from(beatmap) as ReturnType<typeof db.select>;

  if (needsBeatmapset) {
    q = q.leftJoin(beatmapset, eq(beatmap.beatmapsetId, beatmapset.id)) as typeof q;
  }
  if (needsRatio) {
    q = q.leftJoin(beatmapManiaRatio, eq(beatmapManiaRatio.beatmapId, beatmap.id)) as typeof q;
  }
  if (needsRating) {
    // INNER JOIN: exclude beatmaps that have no row for this rating type
    q = q.innerJoin(
      beatmapRating,
      and(eq(beatmapRating.beatmapId, beatmap.id), eq(beatmapRating.ratingType, filter!.rating!.type)),
    ) as typeof q;
    if (needsSkill) {
      q = q.leftJoin(beatmapManiaSkill, eq(beatmapManiaSkill.ratingId, beatmapRating.id)) as typeof q;
    }
  }

  // Count
  // deno-lint-ignore no-explicit-any
  const countQ = (q as any).then ? q : q;
  const [{ total }] = await db
    .select({ total: count() })
    .from(beatmap)
    .$dynamic()
    .where(where);

  // Items
  // deno-lint-ignore no-explicit-any
  const rows = await (q as any).where(where).orderBy(...order).limit(limit).offset(offset);

  return {
    items: rows.map((r: { beatmap: typeof beatmap.$inferSelect }) => r.beatmap),
    total: Number(total),
  };
}

export function getBeatmap(
  db: typeof DB,
  opts: { id?: number | null; osuId?: number | null },
) {
  if (opts.id != null) {
    return db.query.beatmap.findFirst({ where: (b, { eq }) => eq(b.id, opts.id!) });
  }
  if (opts.osuId != null) {
    return db.query.beatmap.findFirst({ where: (b, { eq }) => eq(b.osuId, opts.osuId!) });
  }
  return null;
}
