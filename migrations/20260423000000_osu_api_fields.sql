-- Additional fields from the osu! API (v2) that were previously discarded.
-- Populated on seeder re-run; safe defaults let existing rows stay valid.

ALTER TABLE beatmap
    ADD COLUMN IF NOT EXISTS play_count          integer       NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS pass_count          integer       NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS difficulty_rating   numeric(5, 2) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS mapper_user_id      integer,
    ADD COLUMN IF NOT EXISTS is_scoreable        boolean       NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS is_convert          boolean       NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS osu_last_updated    timestamp,
    ADD COLUMN IF NOT EXISTS osu_ranked_code     smallint;

CREATE INDEX IF NOT EXISTS idx_beatmap_play_count          ON beatmap (play_count DESC);
CREATE INDEX IF NOT EXISTS idx_beatmap_difficulty_rating   ON beatmap (difficulty_rating DESC);

ALTER TABLE beatmapset
    ADD COLUMN IF NOT EXISTS mapper_user_id   integer,
    ADD COLUMN IF NOT EXISTS play_count       bigint   NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS favourite_count  integer  NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS submitted_at     timestamp,
    ADD COLUMN IF NOT EXISTS ranked_at        timestamp,
    ADD COLUMN IF NOT EXISTS language         varchar(40),
    ADD COLUMN IF NOT EXISTS genre            varchar(40),
    ADD COLUMN IF NOT EXISTS set_bpm          numeric(10, 2);

CREATE INDEX IF NOT EXISTS idx_beatmapset_play_count       ON beatmapset (play_count DESC);
CREATE INDEX IF NOT EXISTS idx_beatmapset_favourite_count  ON beatmapset (favourite_count DESC);
CREATE INDEX IF NOT EXISTS idx_beatmapset_ranked_at        ON beatmapset (ranked_at DESC);
