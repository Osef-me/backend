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

type Row = Record<string, unknown>;

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

type HistBucket = { bucket: string; count: number; order: number };

type RatingTypeStat = {
  type: string;
  count: number;
  min: number;
  max: number;
  avg: number;
  p50: number;
  p95: number;
  histogram: HistBucket[];
  top: RatingTop | null;
};

type RatioRadar = {
  keys: number;
  stream: number;
  jumpstream: number;
  handstream: number;
  stamina: number;
  jackspeed: number;
  chordjack: number;
  technical: number;
  count: number;
};

const mapRef = (r: Row): MapRef => ({
  id: Number(r.id),
  osuId: r.osu_id == null ? null : Number(r.osu_id),
  difficulty: String(r.difficulty),
  artist: String(r.artist),
  title: String(r.title),
  creator: String(r.creator),
  coverUrl: r.cover_url == null ? null : String(r.cover_url),
});

export async function getStats() {
  const [
    totals,
    keyModes,
    statuses,
    bpmAgg,
    bpmHist,
    odHist,
    hpHist,
    lengthAgg,
    lengthBuckets,
    notesHist,
    densityHist,
    ratingTypes,
    ratingHistograms,
    ratioByKeys,
    activityDeltas,
    activityCalendar,
    growthSparkline,
    topCreators,
    topMappersByNotes,
    topArtists,
    newestSets,
    featureFlags,
    tagCloud,
  ] = await Promise.all([
    db.execute(sql`
      SELECT
        (SELECT count(*)::int FROM beatmap)             AS beatmaps,
        (SELECT count(*)::int FROM beatmapset)          AS beatmapsets,
        (SELECT count(*)::int FROM beatmap_rating)      AS ratings,
        (SELECT count(*)::int FROM beatmap_mania_skill) AS skills,
        (SELECT count(*)::int FROM beatmap_duplicate)   AS duplicates
    `),
    db.execute(sql`
      SELECT cs::int AS keys, count(*)::int AS count
      FROM beatmap GROUP BY cs ORDER BY cs
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
      SELECT bucket_label, bucket_order, count(*)::int AS count
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
          WHEN bpm < 100 THEN 0 WHEN bpm < 140 THEN 1 WHEN bpm < 160 THEN 2
          WHEN bpm < 180 THEN 3 WHEN bpm < 200 THEN 4 WHEN bpm < 220 THEN 5
          WHEN bpm < 260 THEN 6 ELSE 7
        END AS bucket_order
        FROM beatmap WHERE bpm BETWEEN 40 AND 1000
      ) s
      GROUP BY bucket_label, bucket_order ORDER BY bucket_order
    `),
    db.execute(sql`
      SELECT od::int::text AS bucket_label, od::int AS bucket_order, count(*)::int AS count
      FROM beatmap GROUP BY od::int ORDER BY od::int
    `),
    db.execute(sql`
      SELECT hp::int::text AS bucket_label, hp::int AS bucket_order, count(*)::int AS count
      FROM beatmap GROUP BY hp::int ORDER BY hp::int
    `),
    db.execute(sql`
      SELECT
        min(drain_time)::int  AS min_drain,
        max(drain_time)::int  AS max_drain,
        round(avg(drain_time))::int  AS avg_drain,
        min(total_time)::int  AS min_total,
        max(total_time)::int  AS max_total,
        round(avg(total_time))::int AS avg_total,
        percentile_cont(0.5) WITHIN GROUP (ORDER BY drain_time)::int AS median_drain
      FROM beatmap
    `),
    db.execute(sql`
      SELECT bucket_label, bucket_order, count(*)::int AS count
      FROM (
        SELECT CASE
          WHEN drain_time < 60  THEN '<1m'
          WHEN drain_time < 120 THEN '1-2m'
          WHEN drain_time < 180 THEN '2-3m'
          WHEN drain_time < 300 THEN '3-5m'
          WHEN drain_time < 600 THEN '5-10m'
          ELSE '10m+'
        END AS bucket_label,
        CASE
          WHEN drain_time < 60 THEN 0 WHEN drain_time < 120 THEN 1
          WHEN drain_time < 180 THEN 2 WHEN drain_time < 300 THEN 3
          WHEN drain_time < 600 THEN 4 ELSE 5
        END AS bucket_order
        FROM beatmap
      ) s
      GROUP BY bucket_label, bucket_order ORDER BY bucket_order
    `),
    db.execute(sql`
      SELECT bucket_label, bucket_order, count(*)::int AS count
      FROM (
        SELECT CASE
          WHEN (count_circles + count_sliders) < 200  THEN '<200'
          WHEN (count_circles + count_sliders) < 500  THEN '200-499'
          WHEN (count_circles + count_sliders) < 1000 THEN '500-999'
          WHEN (count_circles + count_sliders) < 2000 THEN '1k-2k'
          WHEN (count_circles + count_sliders) < 4000 THEN '2k-4k'
          WHEN (count_circles + count_sliders) < 8000 THEN '4k-8k'
          ELSE '8k+'
        END AS bucket_label,
        CASE
          WHEN (count_circles + count_sliders) < 200 THEN 0
          WHEN (count_circles + count_sliders) < 500 THEN 1
          WHEN (count_circles + count_sliders) < 1000 THEN 2
          WHEN (count_circles + count_sliders) < 2000 THEN 3
          WHEN (count_circles + count_sliders) < 4000 THEN 4
          WHEN (count_circles + count_sliders) < 8000 THEN 5
          ELSE 6
        END AS bucket_order
        FROM beatmap
      ) s
      GROUP BY bucket_label, bucket_order ORDER BY bucket_order
    `),
    db.execute(sql`
      SELECT bucket_label, bucket_order, count(*)::int AS count
      FROM (
        SELECT CASE
          WHEN drain_time > 0 THEN
            CASE
              WHEN (count_circles+count_sliders)::float / drain_time < 5  THEN '0-5'
              WHEN (count_circles+count_sliders)::float / drain_time < 10 THEN '5-10'
              WHEN (count_circles+count_sliders)::float / drain_time < 15 THEN '10-15'
              WHEN (count_circles+count_sliders)::float / drain_time < 20 THEN '15-20'
              WHEN (count_circles+count_sliders)::float / drain_time < 25 THEN '20-25'
              WHEN (count_circles+count_sliders)::float / drain_time < 30 THEN '25-30'
              WHEN (count_circles+count_sliders)::float / drain_time < 35 THEN '30-35'
              WHEN (count_circles+count_sliders)::float / drain_time < 40 THEN '35-40'
              WHEN (count_circles+count_sliders)::float / drain_time < 50 THEN '40-50'
              WHEN (count_circles+count_sliders)::float / drain_time < 60 THEN '50-60'
              ELSE '60+'
            END
          ELSE '0-5'
        END AS bucket_label,
        CASE
          WHEN drain_time > 0 THEN
            CASE
              WHEN (count_circles+count_sliders)::float / drain_time < 5  THEN 0
              WHEN (count_circles+count_sliders)::float / drain_time < 10 THEN 1
              WHEN (count_circles+count_sliders)::float / drain_time < 15 THEN 2
              WHEN (count_circles+count_sliders)::float / drain_time < 20 THEN 3
              WHEN (count_circles+count_sliders)::float / drain_time < 25 THEN 4
              WHEN (count_circles+count_sliders)::float / drain_time < 30 THEN 5
              WHEN (count_circles+count_sliders)::float / drain_time < 35 THEN 6
              WHEN (count_circles+count_sliders)::float / drain_time < 40 THEN 7
              WHEN (count_circles+count_sliders)::float / drain_time < 50 THEN 8
              WHEN (count_circles+count_sliders)::float / drain_time < 60 THEN 9
              ELSE 10
            END
          ELSE 0
        END AS bucket_order
        FROM beatmap
      ) s
      GROUP BY bucket_label, bucket_order ORDER BY bucket_order
    `),
    db.execute(sql`
      SELECT
        rating_type AS type,
        count(*)::int AS count,
        min(rating)::float AS min,
        max(rating)::float AS max,
        round(avg(rating)::numeric, 2)::float AS avg,
        percentile_cont(0.5) WITHIN GROUP (ORDER BY rating)::float AS p50,
        percentile_cont(0.95) WITHIN GROUP (ORDER BY rating)::float AS p95
      FROM beatmap_rating
      GROUP BY rating_type
      ORDER BY type
    `),
    db.execute(sql`
      -- Buckets scaled per rating type:
      -- osu*/sunnyxxy 0..10 step 1 then 10+
      -- interlude     0..18 step 2 then 18+
      -- etterna       0..40 step 5 then 40+
      -- quaver        0..50 step 5 then 50+
      SELECT rating_type, bucket_order, bucket_label, count(*)::int AS count
      FROM (
        SELECT rating_type,
          CASE
            WHEN rating_type IN ('osu2016','osu2018','osu_current','sunnyxxy') THEN
              LEAST(10, floor(GREATEST(rating, 0))::int)
            WHEN rating_type = 'interlude' THEN
              LEAST(9, floor(GREATEST(rating, 0) / 2)::int)
            WHEN rating_type = 'etterna' THEN
              LEAST(8, floor(GREATEST(rating, 0) / 5)::int)
            ELSE -- quaver
              LEAST(10, floor(GREATEST(rating, 0) / 5)::int)
          END AS bucket_order,
          CASE
            WHEN rating_type IN ('osu2016','osu2018','osu_current','sunnyxxy') THEN
              CASE
                WHEN rating < 1  THEN '0-1'  WHEN rating < 2  THEN '1-2'
                WHEN rating < 3  THEN '2-3'  WHEN rating < 4  THEN '3-4'
                WHEN rating < 5  THEN '4-5'  WHEN rating < 6  THEN '5-6'
                WHEN rating < 7  THEN '6-7'  WHEN rating < 8  THEN '7-8'
                WHEN rating < 9  THEN '8-9'  WHEN rating < 10 THEN '9-10'
                ELSE '10+'
              END
            WHEN rating_type = 'interlude' THEN
              CASE
                WHEN rating < 2  THEN '0-2'   WHEN rating < 4  THEN '2-4'
                WHEN rating < 6  THEN '4-6'   WHEN rating < 8  THEN '6-8'
                WHEN rating < 10 THEN '8-10'  WHEN rating < 12 THEN '10-12'
                WHEN rating < 14 THEN '12-14' WHEN rating < 16 THEN '14-16'
                WHEN rating < 18 THEN '16-18' ELSE '18+'
              END
            WHEN rating_type = 'etterna' THEN
              CASE
                WHEN rating < 5  THEN '0-5'   WHEN rating < 10 THEN '5-10'
                WHEN rating < 15 THEN '10-15' WHEN rating < 20 THEN '15-20'
                WHEN rating < 25 THEN '20-25' WHEN rating < 30 THEN '25-30'
                WHEN rating < 35 THEN '30-35' WHEN rating < 40 THEN '35-40'
                ELSE '40+'
              END
            ELSE -- quaver
              CASE
                WHEN rating < 5  THEN '0-5'   WHEN rating < 10 THEN '5-10'
                WHEN rating < 15 THEN '10-15' WHEN rating < 20 THEN '15-20'
                WHEN rating < 25 THEN '20-25' WHEN rating < 30 THEN '25-30'
                WHEN rating < 35 THEN '30-35' WHEN rating < 40 THEN '35-40'
                WHEN rating < 45 THEN '40-45' WHEN rating < 50 THEN '45-50'
                ELSE '50+'
              END
          END AS bucket_label
        FROM beatmap_rating
      ) s
      GROUP BY rating_type, bucket_order, bucket_label
      ORDER BY rating_type, bucket_order
    `),
    db.execute(sql`
      SELECT
        bm.cs::int AS keys,
        count(*)::int AS count,
        round(avg(r.stream)::numeric,     4)::float AS stream,
        round(avg(r.jumpstream)::numeric, 4)::float AS jumpstream,
        round(avg(r.handstream)::numeric, 4)::float AS handstream,
        round(avg(r.stamina)::numeric,    4)::float AS stamina,
        round(avg(r.jackspeed)::numeric,  4)::float AS jackspeed,
        round(avg(r.chordjack)::numeric,  4)::float AS chordjack,
        round(avg(r.technical)::numeric,  4)::float AS technical
      FROM beatmap_mania_ratio r
      JOIN beatmap bm ON bm.id = r.beatmap_id
      GROUP BY bm.cs
      ORDER BY bm.cs
    `),
    db.execute(sql`
      SELECT
        (SELECT count(*) FILTER (WHERE created_at >= now() - interval '24 hours')::int FROM beatmap) AS d1,
        (SELECT count(*) FILTER (WHERE created_at >= now() - interval '7 days')::int   FROM beatmap) AS d7,
        (SELECT count(*) FILTER (WHERE created_at >= now() - interval '30 days')::int  FROM beatmap) AS d30,
        (SELECT count(*) FILTER (WHERE osu_status_changed_at >= now() - interval '30 days')::int FROM beatmapset) AS ranked30,
        (SELECT min(created_at)::text FROM beatmap) AS first_seen,
        (SELECT max(created_at)::text FROM beatmap) AS last_seen
    `),
    db.execute(sql`
      SELECT to_char(date_trunc('day', osu_status_changed_at), 'YYYY-MM-DD') AS day,
             count(*)::int AS count
      FROM beatmapset
      WHERE osu_status_changed_at IS NOT NULL
        AND osu_status_changed_at >= now() - interval '365 days'
      GROUP BY day
      ORDER BY day
    `),
    db.execute(sql`
      SELECT to_char(date_trunc('day', created_at), 'YYYY-MM-DD') AS day,
             count(*)::int AS count
      FROM beatmap
      WHERE created_at >= now() - interval '30 days'
      GROUP BY day
      ORDER BY day
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
      SELECT bs.creator,
             sum(bm.count_circles + bm.count_sliders)::bigint AS total_notes,
             count(*)::int AS maps
      FROM beatmap bm
      JOIN beatmapset bs ON bs.id = bm.beatmapset_id
      GROUP BY bs.creator
      ORDER BY total_notes DESC
      LIMIT 10
    `),
    db.execute(sql`
      SELECT artist, count(*)::int AS count
      FROM beatmapset GROUP BY artist ORDER BY count DESC LIMIT 10
    `),
    db.execute(sql`
      SELECT id, osu_id, artist, title, creator, cover_url, osu_status_changed_at
      FROM beatmapset
      WHERE osu_status_changed_at IS NOT NULL
      ORDER BY osu_status_changed_at DESC
      LIMIT 8
    `),
    db.execute(sql`
      SELECT
        (count(*) FILTER (WHERE has_video))::float      / NULLIF(count(*),0) AS video,
        (count(*) FILTER (WHERE has_storyboard))::float / NULLIF(count(*),0) AS storyboard,
        (count(*) FILTER (WHERE is_explicit))::float    / NULLIF(count(*),0) AS explicit,
        count(*)::int AS total,
        count(*) FILTER (WHERE has_video)::int      AS video_count,
        count(*) FILTER (WHERE has_storyboard)::int AS sb_count,
        count(*) FILTER (WHERE is_explicit)::int    AS expl_count
      FROM beatmapset
    `),
    db.execute(sql`
      SELECT tag, count(*)::int AS count
      FROM (SELECT unnest(tags) AS tag FROM beatmapset WHERE tags IS NOT NULL) s
      WHERE length(tag) BETWEEN 2 AND 40
      GROUP BY tag
      ORDER BY count DESC
      LIMIT 60
    `),
  ]);

  // rating top per type
  const ratingTypeTops = await Promise.all(
    (ratingTypes as unknown as Row[]).map(async (rt) => {
      const rows = await db.execute(sql`
        SELECT bm.id, bm.osu_id, bm.difficulty,
               bs.artist, bs.title, bs.creator, bs.cover_url,
               br.rating::float AS rating
        FROM beatmap_rating br
        JOIN beatmap bm    ON bm.id = br.beatmap_id
        JOIN beatmapset bs ON bs.id = bm.beatmapset_id
        WHERE br.rating_type = ${String(rt.type)}
        ORDER BY br.rating DESC
        LIMIT 1
      `);
      const row = (rows as unknown as Row[])[0];
      return { type: String(rt.type), row };
    }),
  );
  const topByRatingType: Record<string, RatingTop | null> = {};
  for (const { type, row } of ratingTypeTops) {
    topByRatingType[type] = row
      ? { ...mapRef(row), rating: Number(row.rating) }
      : null;
  }

  // top per skill
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
      return { skill, row: (rows as unknown as Row[])[0] };
    }),
  );
  const skillTop: Record<string, SkillTop> = {};
  for (const { skill, row } of skillTopRows) {
    if (!row) continue;
    skillTop[skill] = { ...mapRef(row), value: Number(row.value) };
  }

  // extremes — only fields with real data (mania, no osu SR / no plays)
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
    (SELECT 'shortest_drain', bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url, bm.drain_time::float
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       WHERE drain_time >= 30
       ORDER BY drain_time ASC LIMIT 1)
    UNION ALL
    (SELECT 'most_notes', bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url,
            (bm.count_circles + bm.count_sliders)::float
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       ORDER BY (bm.count_circles + bm.count_sliders) DESC LIMIT 1)
    UNION ALL
    (SELECT 'max_combo', bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url, bm.max_combo::float
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       ORDER BY bm.max_combo DESC LIMIT 1)
    UNION ALL
    (SELECT 'highest_density', bm.id, bm.osu_id, bm.difficulty, bs.artist, bs.title, bs.creator, bs.cover_url,
            ((bm.count_circles + bm.count_sliders)::float / NULLIF(bm.drain_time, 0))
       FROM beatmap bm JOIN beatmapset bs ON bs.id = bm.beatmapset_id
       WHERE bm.drain_time >= 60
         AND (bm.count_circles + bm.count_sliders) <= 100000
       ORDER BY (bm.count_circles + bm.count_sliders)::float / NULLIF(bm.drain_time, 0) DESC NULLS LAST
       LIMIT 1)
  `);
  const extremesMap: Record<string, (MapRef & { value: number }) | null> = {
    fastest_bpm: null,
    longest_drain: null,
    shortest_drain: null,
    most_notes: null,
    max_combo: null,
    highest_density: null,
  };
  for (const r of extremes as unknown as Row[]) {
    extremesMap[String(r.kind)] = { ...mapRef(r), value: Number(r.v) };
  }

  const rtHistograms = new Map<string, HistBucket[]>();
  for (const r of ratingHistograms as unknown as Row[]) {
    const t = String(r.rating_type);
    if (!rtHistograms.has(t)) rtHistograms.set(t, []);
    rtHistograms.get(t)!.push({
      bucket: String(r.bucket_label),
      count: Number(r.count),
      order: Number(r.bucket_order),
    });
  }

  const rtStats: RatingTypeStat[] =
    (ratingTypes as unknown as Row[]).map((r) => ({
      type: String(r.type),
      count: Number(r.count),
      min: Number(r.min),
      max: Number(r.max),
      avg: Number(r.avg),
      p50: Number(r.p50),
      p95: Number(r.p95),
      histogram: rtHistograms.get(String(r.type)) ?? [],
      top: topByRatingType[String(r.type)] ?? null,
    }));

  const ratioRadars: RatioRadar[] =
    (ratioByKeys as unknown as Row[]).map((r) => ({
      keys: Number(r.keys),
      count: Number(r.count),
      stream: Number(r.stream),
      jumpstream: Number(r.jumpstream),
      handstream: Number(r.handstream),
      stamina: Number(r.stamina),
      jackspeed: Number(r.jackspeed),
      chordjack: Number(r.chordjack),
      technical: Number(r.technical),
    }));

  const [t] = totals as unknown as Row[];
  const [la] = lengthAgg as unknown as Row[];
  const [bpm] = bpmAgg as unknown as Row[];
  const [act] = activityDeltas as unknown as Row[];
  const [ff] = featureFlags as unknown as Row[];

  const mapHist = (rows: unknown): HistBucket[] =>
    (rows as Row[]).map((r) => ({
      bucket: String(r.bucket_label),
      count: Number(r.count),
      order: Number(r.bucket_order),
    }));

  return {
    totals: {
      beatmaps: Number(t.beatmaps),
      beatmapsets: Number(t.beatmapsets),
      ratings: Number(t.ratings),
      skills: Number(t.skills),
      duplicates: Number(t.duplicates),
    },
    activity: {
      d1: Number(act.d1),
      d7: Number(act.d7),
      d30: Number(act.d30),
      ranked30: Number(act.ranked30),
      firstSeen: act.first_seen ? String(act.first_seen) : null,
      lastSeen: act.last_seen ? String(act.last_seen) : null,
      calendar: (activityCalendar as unknown as Row[]).map((r) => ({
        day: String(r.day),
        count: Number(r.count),
      })),
      sparkline: (growthSparkline as unknown as Row[]).map((r) => ({
        day: String(r.day),
        count: Number(r.count),
      })),
    },
    keyModes: (keyModes as unknown as Row[]).map((r) => ({
      keys: Number(r.keys),
      count: Number(r.count),
    })),
    statuses: (statuses as unknown as Row[]).map((r) => ({
      status: String(r.status),
      count: Number(r.count),
    })),
    bpm: {
      min: Number(bpm.min),
      max: Number(bpm.max),
      avg: Number(bpm.avg),
      median: Number(bpm.median),
      histogram: mapHist(bpmHist),
    },
    od: { histogram: mapHist(odHist) },
    hp: { histogram: mapHist(hpHist) },
    length: {
      minDrain: Number(la.min_drain),
      maxDrain: Number(la.max_drain),
      avgDrain: Number(la.avg_drain),
      medianDrain: Number(la.median_drain),
      minTotal: Number(la.min_total),
      maxTotal: Number(la.max_total),
      avgTotal: Number(la.avg_total),
      buckets: mapHist(lengthBuckets),
    },
    notes: {
      histogram: mapHist(notesHist),
      densityHistogram: mapHist(densityHist),
    },
    ratingTypes: rtStats,
    ratioByKeys: ratioRadars,
    skillTop: SKILLS.map((s) => ({ skill: s, top: skillTop[s] ?? null })),
    topCreators: (topCreators as unknown as Row[]).map((r) => ({
      creator: String(r.creator),
      count: Number(r.count),
    })),
    topMappersByNotes: (topMappersByNotes as unknown as Row[]).map((r) => ({
      creator: String(r.creator),
      totalNotes: Number(r.total_notes),
      maps: Number(r.maps),
    })),
    topArtists: (topArtists as unknown as Row[]).map((r) => ({
      artist: String(r.artist),
      count: Number(r.count),
    })),
    newestRanked: (newestSets as unknown as Row[]).map((r) => ({
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
    features: {
      video: Number(ff.video ?? 0),
      storyboard: Number(ff.storyboard ?? 0),
      explicit: Number(ff.explicit ?? 0),
      total: Number(ff.total),
      videoCount: Number(ff.video_count),
      storyboardCount: Number(ff.sb_count),
      explicitCount: Number(ff.expl_count),
    },
    tagCloud: (tagCloud as unknown as Row[]).map((r) => ({
      tag: String(r.tag),
      count: Number(r.count),
    })),
    extremes: extremesMap,
    generatedAt: new Date().toISOString(),
  };
}
