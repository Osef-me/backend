import { sql } from "drizzle-orm";
import { db } from "../db/client.ts";

const SKILLS = [
  "stream",
  "jumpstream",
  "handstream",
  "stamina",
  "jackspeed",
  "chordjack",
  "technical",
] as const;

type MapRef = {
  id: number;
  osuId: number | null;
  difficulty: string;
  artist: string;
  title: string;
  creator: string;
  coverUrl: string | null;
};

type SkillTop = MapRef & { value: number };
type RatingTop = MapRef & { rating: number };

type RatingTypeStat = {
  type: string;
  count: number;
  min: number;
  max: number;
  avg: number;
  top: RatingTop | null;
};

export async function getStats() {
  const [
    totals,
    modes,
    statuses,
    bpmAgg,
    bpmHist,
    lengthAgg,
    ratingTypes,
    topCreators,
    topArtists,
    newestSets,
  ] = await Promise.all([
    db.execute(sql`
      SELECT
        (SELECT count(*)::int FROM beatmap)             AS beatmaps,
        (SELECT count(*)::int FROM beatmapset)          AS beatmapsets,
        (SELECT count(*)::int FROM beatmap_rating)      AS ratings,
        (SELECT count(*)::int FROM beatmap_mania_skill) AS skills,
        (SELECT count(*)::int FROM beatmap_duplicate)   AS duplicates,
        (SELECT count(*)::int FROM pending_beatmap)     AS pending,
        (SELECT count(*)::int FROM failed_query)        AS failed
    `),
    db.execute(sql`
      SELECT mode, count(*)::int AS count
      FROM beatmap GROUP BY mode ORDER BY count DESC
    `),
    db.execute(sql`
      SELECT status, count(*)::int AS count
      FROM beatmap GROUP BY status ORDER BY count DESC
    `),
    db.execute(sql`
      SELECT
        min(bpm)::float              AS min,
        max(bpm)::float              AS max,
        round(avg(bpm)::numeric, 2)  AS avg,
        percentile_cont(0.5) WITHIN GROUP (ORDER BY bpm)::float AS median
      FROM beatmap
      WHERE bpm BETWEEN 40 AND 1000
    `),
    db.execute(sql`
      SELECT bucket_label, count(*)::int AS count
      FROM (
        SELECT CASE
          WHEN bpm < 100 THEN '<100'
          WHEN bpm < 140 THEN '100-139'
          WHEN bpm < 160 THEN '140-159'
          WHEN bpm < 180 THEN '160-179'
          WHEN bpm < 200 THEN '180-199'
          WHEN bpm < 220 THEN '200-219'
          WHEN bpm < 260 THEN '220-259'
          ELSE '260+'
        END AS bucket_label,
        CASE
          WHEN bpm < 100 THEN 0
          WHEN bpm < 140 THEN 1
          WHEN bpm < 160 THEN 2
          WHEN bpm < 180 THEN 3
          WHEN bpm < 200 THEN 4
          WHEN bpm < 220 THEN 5
          WHEN bpm < 260 THEN 6
          ELSE 7
        END AS bucket_order
        FROM beatmap
        WHERE bpm BETWEEN 40 AND 1000
      ) s
      GROUP BY bucket_label, bucket_order
      ORDER BY bucket_order
    `),
    db.execute(sql`
      SELECT
        min(drain_time)::int  AS min_drain,
        max(drain_time)::int  AS max_drain,
        round(avg(drain_time))::int  AS avg_drain,
        min(total_time)::int  AS min_total,
        max(total_time)::int  AS max_total,
        round(avg(total_time))::int AS avg_total
      FROM beatmap
    `),
    db.execute(sql`
      SELECT
        rating_type AS type,
        count(*)::int AS count,
        min(rating)::float AS min,
        max(rating)::float AS max,
        round(avg(rating)::numeric, 2)::float AS avg
      FROM beatmap_rating
      GROUP BY rating_type
      ORDER BY type
    `),
    db.execute(sql`
      SELECT bs.creator, count(*)::int AS count
      FROM beatmap bm
      JOIN beatmapset bs ON bs.id = bm.beatmapset_id
      GROUP BY bs.creator
      ORDER BY count DESC
      LIMIT 10
    `),
    db.execute(sql`
      SELECT artist, count(*)::int AS count
      FROM beatmapset
      GROUP BY artist
      ORDER BY count DESC
      LIMIT 10
    `),
    db.execute(sql`
      SELECT id, osu_id, artist, title, creator, cover_url,
             osu_status_changed_at
      FROM beatmapset
      WHERE osu_status_changed_at IS NOT NULL
      ORDER BY osu_status_changed_at DESC
      LIMIT 5
    `),
  ]);

  // top per rating type (one query per type — indexed, each < 50ms)
  const ratingTypeTops = await Promise.all(
    (ratingTypes as unknown as { type: string }[]).map(async (rt) => {
      const rows = await db.execute(sql`
        SELECT bm.id, bm.osu_id, bm.difficulty,
               bs.artist, bs.title, bs.creator, bs.cover_url,
               br.rating::float AS rating
        FROM beatmap_rating br
        JOIN beatmap bm    ON bm.id = br.beatmap_id
        JOIN beatmapset bs ON bs.id = bm.beatmapset_id
        WHERE br.rating_type = ${rt.type}
        ORDER BY br.rating DESC
        LIMIT 1
      `);
      const row = (rows as unknown as Record<string, unknown>[])[0];
      return { type: rt.type, row };
    }),
  );

  const topByRatingType: Record<string, RatingTop | null> = {};
  for (const { type, row } of ratingTypeTops) {
    topByRatingType[type] = row ? rowToRatingTop(row) : null;
  }

  const skillTopRows = await Promise.all(
    SKILLS.map(async (skill) => {
      const col = sql.identifier(skill);
      const rows = await db.execute(sql`
        SELECT bm.id, bm.osu_id, bm.difficulty,
               bs.artist, bs.title, bs.creator, bs.cover_url,
               bms.${col}::float AS value
        FROM beatmap_mania_skill bms
        JOIN beatmap_rating br ON br.id = bms.rating_id
        JOIN beatmap bm        ON bm.id = br.beatmap_id
        JOIN beatmapset bs     ON bs.id = bm.beatmapset_id
        WHERE bms.${col} IS NOT NULL
        ORDER BY bms.${col} DESC
        LIMIT 1
      `);
      return { skill, row: (rows as unknown as Record<string, unknown>[])[0] };
    }),
  );

  const skillTop: Record<string, SkillTop> = {};
  for (const { skill, row } of skillTopRows) {
    if (!row) continue;
    skillTop[skill] = {
      id: Number(row.id),
      osuId: row.osu_id == null ? null : Number(row.osu_id),
      difficulty: String(row.difficulty),
      artist: String(row.artist),
      title: String(row.title),
      creator: String(row.creator),
      coverUrl: row.cover_url == null ? null : String(row.cover_url),
      value: Number(row.value),
    };
  }

  const extremes = await db.execute(sql`
    (SELECT 'fastest_bpm' AS kind, bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url, bm.bpm::float AS v
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       WHERE bpm BETWEEN 40 AND 1000
       ORDER BY bpm DESC LIMIT 1)
    UNION ALL
    (SELECT 'longest_drain', bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url, bm.drain_time::float
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       ORDER BY drain_time DESC LIMIT 1)
    UNION ALL
    (SELECT 'most_notes', bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url,
            (bm.count_circles + bm.count_sliders)::float
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       ORDER BY (bm.count_circles + bm.count_sliders) DESC LIMIT 1)
    UNION ALL
    (SELECT 'max_combo', bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url, bm.max_combo::float
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       ORDER BY bm.max_combo DESC LIMIT 1)
  `);

  const extremesMap: Record<string, (MapRef & { value: number }) | null> = {
    fastest_bpm: null,
    longest_drain: null,
    most_notes: null,
    max_combo: null,
  };
  for (const r of extremes as unknown as Record<string, unknown>[]) {
    const k = String(r.kind);
    extremesMap[k] = {
      id: Number(r.id),
      osuId: r.osu_id == null ? null : Number(r.osu_id),
      difficulty: String(r.difficulty),
      artist: String(r.artist),
      title: String(r.title),
      creator: String(r.creator),
      coverUrl: r.cover_url == null ? null : String(r.cover_url),
      value: Number(r.v),
    };
  }

  const rtStats: RatingTypeStat[] =
    (ratingTypes as unknown as Record<string, unknown>[]).map((r) => ({
      type: String(r.type),
      count: Number(r.count),
      min: Number(r.min),
      max: Number(r.max),
      avg: Number(r.avg),
      top: topByRatingType[String(r.type)] ?? null,
    }));

  const [t] = totals as unknown as Record<string, unknown>[];
  const [la] = lengthAgg as unknown as Record<string, unknown>[];
  const [bpm] = bpmAgg as unknown as Record<string, unknown>[];

  return {
    totals: {
      beatmaps: Number(t.beatmaps),
      beatmapsets: Number(t.beatmapsets),
      ratings: Number(t.ratings),
      skills: Number(t.skills),
      duplicates: Number(t.duplicates),
      pending: Number(t.pending),
      failed: Number(t.failed),
    },
    modes: (modes as unknown as Record<string, unknown>[]).map((r) => ({
      mode: Number(r.mode),
      count: Number(r.count),
    })),
    statuses: (statuses as unknown as Record<string, unknown>[]).map((r) => ({
      status: String(r.status),
      count: Number(r.count),
    })),
    bpm: {
      min: Number(bpm.min),
      max: Number(bpm.max),
      avg: Number(bpm.avg),
      median: Number(bpm.median),
      histogram: (bpmHist as unknown as Record<string, unknown>[]).map((r) => ({
        bucket: String(r.bucket_label),
        count: Number(r.count),
      })),
    },
    length: {
      minDrain: Number(la.min_drain),
      maxDrain: Number(la.max_drain),
      avgDrain: Number(la.avg_drain),
      minTotal: Number(la.min_total),
      maxTotal: Number(la.max_total),
      avgTotal: Number(la.avg_total),
    },
    ratingTypes: rtStats,
    skillTop: SKILLS.map((s) => ({ skill: s, top: skillTop[s] ?? null })),
    topCreators: (topCreators as unknown as Record<string, unknown>[]).map((r) => ({
      creator: String(r.creator),
      count: Number(r.count),
    })),
    topArtists: (topArtists as unknown as Record<string, unknown>[]).map((r) => ({
      artist: String(r.artist),
      count: Number(r.count),
    })),
    newestRanked: (newestSets as unknown as Record<string, unknown>[]).map((r) => ({
      id: Number(r.id),
      osuId: r.osu_id == null ? null : Number(r.osu_id),
      artist: String(r.artist),
      title: String(r.title),
      creator: String(r.creator),
      coverUrl: r.cover_url == null ? null : String(r.cover_url),
      rankedAt: r.osu_status_changed_at
        ? new Date(r.osu_status_changed_at as string).toISOString()
        : null,
    })),
    extremes: extremesMap,
    generatedAt: new Date().toISOString(),
  };
}

function rowToRatingTop(r: Record<string, unknown>): RatingTop {
  return {
    id: Number(r.id),
    osuId: r.osu_id == null ? null : Number(r.osu_id),
    difficulty: String(r.difficulty),
    artist: String(r.artist),
    title: String(r.title),
    creator: String(r.creator),
    coverUrl: r.cover_url == null ? null : String(r.cover_url),
    rating: Number(r.rating),
  };
}
