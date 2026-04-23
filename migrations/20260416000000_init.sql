-- osef-me-v2 — single init migration
-- PostgreSQL — beatmap discovery only

CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================
-- beatmapset
-- ============================================================
CREATE TABLE IF NOT EXISTS beatmapset (
    id                    INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    osu_id                INTEGER UNIQUE,
    artist                VARCHAR(255) NOT NULL,
    artist_unicode        VARCHAR(255),
    title                 VARCHAR(255) NOT NULL,
    title_unicode         VARCHAR(255),
    creator               VARCHAR(255) NOT NULL,
    source                VARCHAR(255),
    tags                  TEXT[],
    has_video             BOOLEAN NOT NULL DEFAULT FALSE,
    has_storyboard        BOOLEAN NOT NULL DEFAULT FALSE,
    is_explicit           BOOLEAN NOT NULL DEFAULT FALSE,
    is_featured           BOOLEAN NOT NULL DEFAULT FALSE,
    cover_url             VARCHAR(255),
    preview_url           VARCHAR(255),
    osu_file_url          VARCHAR(255),
    osu_status_changed_at TIMESTAMP,
    created_at            TIMESTAMP DEFAULT NOW(),
    updated_at            TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_beatmapset_artist_trgm  ON beatmapset USING GIN (artist  gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_beatmapset_title_trgm   ON beatmapset USING GIN (title   gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_beatmapset_creator_trgm ON beatmapset USING GIN (creator gin_trgm_ops);

-- ============================================================
-- beatmap (canonical — unique by notes_hash)
-- notes_hash: normalized at 100 bpm for dupe detection
-- ==================https://media.discordapp.net/attachments/1457874634216308882/1494813398880354315/image.png?ex=69e3f8f6&is=69e2a776&hm=88abbaa8351114e865a99d36c93961fe7be431aef3b59ab5b2de2d23aac4c8e4&=&format=webp&quality=lossless&width=1170&height=544==========================================
CREATE TABLE IF NOT EXISTS beatmap (
    id             INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    osu_id         INTEGER UNIQUE,
    beatmapset_id  INTEGER REFERENCES beatmapset(id) ON DELETE CASCADE,
    difficulty     VARCHAR(255) NOT NULL,
    osu_hash       VARCHAR(128) NOT NULL UNIQUE,
    notes_hash     VARCHAR(128) NOT NULL UNIQUE,
    count_circles  INTEGER NOT NULL,
    count_sliders  INTEGER NOT NULL,
    count_spinners INTEGER NOT NULL,
    max_combo      INTEGER NOT NULL,
    main_pattern   JSONB NOT NULL,
    drain_time     INTEGER NOT NULL,
    total_time     INTEGER NOT NULL,
    bpm            DECIMAL(10,2) NOT NULL,
    cs             DECIMAL(3,1) NOT NULL,
    ar             DECIMAL(3,1) NOT NULL,
    od             DECIMAL(3,1) NOT NULL,
    hp             DECIMAL(3,1) NOT NULL,
    mode           SMALLINT NOT NULL DEFAULT 3,
    status         VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at     TIMESTAMP DEFAULT NOW(),
    updated_at     TIMESTAMP DEFAULT NOW(),
    CONSTRAINT valid_mode   CHECK (mode IN (0, 1, 2, 3)),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'ranked', 'qualified', 'loved', 'graveyard'))
);

CREATE INDEX IF NOT EXISTS idx_beatmap_beatmapset_id    ON beatmap(beatmapset_id);
CREATE INDEX IF NOT EXISTS idx_beatmap_mode             ON beatmap(mode);
CREATE INDEX IF NOT EXISTS idx_beatmap_status           ON beatmap(status);
CREATE INDEX IF NOT EXISTS idx_beatmap_bpm              ON beatmap(bpm);
CREATE INDEX IF NOT EXISTS idx_beatmap_total_time       ON beatmap(total_time);
CREATE INDEX IF NOT EXISTS idx_beatmap_drain_time       ON beatmap(drain_time);
CREATE INDEX IF NOT EXISTS idx_beatmap_od               ON beatmap(od);
CREATE INDEX IF NOT EXISTS idx_beatmap_cs               ON beatmap(cs);
CREATE INDEX IF NOT EXISTS idx_beatmap_main_pattern_gin ON beatmap USING GIN (main_pattern);

-- ============================================================
-- beatmap_duplicate
-- Lean mapping: osu_hash → canonical beatmap.
-- All display info lives on the canonical — no data duplication.
-- ============================================================
CREATE TABLE IF NOT EXISTS beatmap_duplicate (
    id                   INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    canonical_beatmap_id INTEGER NOT NULL REFERENCES beatmap(id) ON DELETE CASCADE,
    osu_id               INTEGER UNIQUE,
    osu_hash             VARCHAR(128) NOT NULL UNIQUE,
    notes_hash           VARCHAR(128) NOT NULL,
    created_at           TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_beatmap_dup_canonical  ON beatmap_duplicate(canonical_beatmap_id);
CREATE INDEX IF NOT EXISTS idx_beatmap_dup_notes_hash ON beatmap_duplicate(notes_hash);

-- ============================================================
-- beatmap_rating (one row per beatmap per rating system)
-- ============================================================
CREATE TABLE IF NOT EXISTS beatmap_rating (
    id          INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    beatmap_id  INTEGER NOT NULL REFERENCES beatmap(id) ON DELETE CASCADE,
    rating      DECIMAL(8,4) NOT NULL CHECK (rating >= 0),
    rating_type VARCHAR(30) NOT NULL,
    created_at  TIMESTAMP DEFAULT NOW(),
    updated_at  TIMESTAMP DEFAULT NOW(),
    UNIQUE (beatmap_id, rating_type),
    CONSTRAINT valid_rating_type CHECK (rating_type IN ('osu2016', 'osu_current', 'etterna', 'quaver', 'interlude', 'sunnyxxy'))
);

CREATE INDEX IF NOT EXISTS idx_beatmap_rating_beatmap_id  ON beatmap_rating(beatmap_id);
CREATE INDEX IF NOT EXISTS idx_beatmap_rating_type_rating ON beatmap_rating(rating_type, rating);

-- ============================================================
-- beatmap_mania_ratio
-- Raw skill ratios (0-1) — property of the map itself, rating-type agnostic.
-- 1:1 with beatmap via UNIQUE(beatmap_id).
-- ============================================================
CREATE TABLE IF NOT EXISTS beatmap_mania_ratio (
    id         INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    beatmap_id INTEGER NOT NULL UNIQUE REFERENCES beatmap(id) ON DELETE CASCADE,
    stream     DECIMAL(5,4),
    jumpstream DECIMAL(5,4),
    handstream DECIMAL(5,4),
    stamina    DECIMAL(5,4),
    jackspeed  DECIMAL(5,4),
    chordjack  DECIMAL(5,4),
    technical  DECIMAL(5,4),
    CONSTRAINT ratio_stream     CHECK (stream     IS NULL OR (stream     >= 0 AND stream     <= 1)),
    CONSTRAINT ratio_jumpstream CHECK (jumpstream IS NULL OR (jumpstream >= 0 AND jumpstream <= 1)),
    CONSTRAINT ratio_handstream CHECK (handstream IS NULL OR (handstream >= 0 AND handstream <= 1)),
    CONSTRAINT ratio_stamina    CHECK (stamina    IS NULL OR (stamina    >= 0 AND stamina    <= 1)),
    CONSTRAINT ratio_jackspeed  CHECK (jackspeed  IS NULL OR (jackspeed  >= 0 AND jackspeed  <= 1)),
    CONSTRAINT ratio_chordjack  CHECK (chordjack  IS NULL OR (chordjack  >= 0 AND chordjack  <= 1)),
    CONSTRAINT ratio_technical  CHECK (technical  IS NULL OR (technical  >= 0 AND technical  <= 1))
);

CREATE INDEX IF NOT EXISTS idx_mania_ratio_stream     ON beatmap_mania_ratio(stream);
CREATE INDEX IF NOT EXISTS idx_mania_ratio_jumpstream ON beatmap_mania_ratio(jumpstream);
CREATE INDEX IF NOT EXISTS idx_mania_ratio_handstream ON beatmap_mania_ratio(handstream);
CREATE INDEX IF NOT EXISTS idx_mania_ratio_stamina    ON beatmap_mania_ratio(stamina);
CREATE INDEX IF NOT EXISTS idx_mania_ratio_jackspeed  ON beatmap_mania_ratio(jackspeed);
CREATE INDEX IF NOT EXISTS idx_mania_ratio_chordjack  ON beatmap_mania_ratio(chordjack);
CREATE INDEX IF NOT EXISTS idx_mania_ratio_technical  ON beatmap_mania_ratio(technical);

-- ============================================================
-- beatmap_mania_skill
-- Precomputed (rating * ratio) per skill, per rating system.
-- App multiplies before insert — queries are direct range scans, no join math.
-- 1:1 with beatmap_rating via UNIQUE(rating_id).
-- ============================================================
CREATE TABLE IF NOT EXISTS beatmap_mania_skill (
    id         INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    rating_id  INTEGER NOT NULL UNIQUE REFERENCES beatmap_rating(id) ON DELETE CASCADE,
    stream     DECIMAL(8,4),
    jumpstream DECIMAL(8,4),
    handstream DECIMAL(8,4),
    stamina    DECIMAL(8,4),
    jackspeed  DECIMAL(8,4),
    chordjack  DECIMAL(8,4),
    technical  DECIMAL(8,4),
    CONSTRAINT valid_stream     CHECK (stream     IS NULL OR stream     >= 0),
    CONSTRAINT valid_jumpstream CHECK (jumpstream IS NULL OR jumpstream >= 0),
    CONSTRAINT valid_handstream CHECK (handstream IS NULL OR handstream >= 0),
    CONSTRAINT valid_stamina    CHECK (stamina    IS NULL OR stamina    >= 0),
    CONSTRAINT valid_jackspeed  CHECK (jackspeed  IS NULL OR jackspeed  >= 0),
    CONSTRAINT valid_chordjack  CHECK (chordjack  IS NULL OR chordjack  >= 0),
    CONSTRAINT valid_technical  CHECK (technical  IS NULL OR technical  >= 0)
);

CREATE INDEX IF NOT EXISTS idx_mania_skill_stream     ON beatmap_mania_skill(stream);
CREATE INDEX IF NOT EXISTS idx_mania_skill_jumpstream ON beatmap_mania_skill(jumpstream);
CREATE INDEX IF NOT EXISTS idx_mania_skill_handstream ON beatmap_mania_skill(handstream);
CREATE INDEX IF NOT EXISTS idx_mania_skill_stamina    ON beatmap_mania_skill(stamina);
CREATE INDEX IF NOT EXISTS idx_mania_skill_jackspeed  ON beatmap_mania_skill(jackspeed);
CREATE INDEX IF NOT EXISTS idx_mania_skill_chordjack  ON beatmap_mania_skill(chordjack);
CREATE INDEX IF NOT EXISTS idx_mania_skill_technical  ON beatmap_mania_skill(technical);

-- ============================================================
-- pending_beatmap (discovery queue, pre-processing)
-- ============================================================
CREATE TABLE IF NOT EXISTS pending_beatmap (
    id         INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    osu_hash   TEXT NOT NULL UNIQUE,
    osu_id     INTEGER,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pending_beatmap_created_at ON pending_beatmap(created_at);

-- ============================================================
-- failed_query (tracks hashes that failed during discovery)
-- ============================================================
CREATE TABLE IF NOT EXISTS failed_query (
    id         INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    hash       TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_failed_query_hash       ON failed_query(hash);
CREATE INDEX IF NOT EXISTS idx_failed_query_created_at ON failed_query(created_at);
