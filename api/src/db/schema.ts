import {
  boolean,
  decimal,
  index,
  integer,
  jsonb,
  pgTable,
  smallint,
  text,
  timestamp,
  unique,
  varchar,
} from "drizzle-orm/pg-core";

export const beatmapset = pgTable(
  "beatmapset",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    osuId: integer("osu_id").unique(),
    artist: varchar("artist", { length: 255 }).notNull(),
    artistUnicode: varchar("artist_unicode", { length: 255 }),
    title: varchar("title", { length: 255 }).notNull(),
    titleUnicode: varchar("title_unicode", { length: 255 }),
    creator: varchar("creator", { length: 255 }).notNull(),
    source: varchar("source", { length: 255 }),
    tags: text("tags").array(),
    hasVideo: boolean("has_video").notNull().default(false),
    hasStoryboard: boolean("has_storyboard").notNull().default(false),
    isExplicit: boolean("is_explicit").notNull().default(false),
    isFeatured: boolean("is_featured").notNull().default(false),
    coverUrl: varchar("cover_url", { length: 255 }),
    previewUrl: varchar("preview_url", { length: 255 }),
    osuFileUrl: varchar("osu_file_url", { length: 255 }),
    osuStatusChangedAt: timestamp("osu_status_changed_at"),
    createdAt: timestamp("created_at").defaultNow(),
    updatedAt: timestamp("updated_at").defaultNow(),
  },
);

export const beatmap = pgTable(
  "beatmap",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    osuId: integer("osu_id").unique(),
    beatmapsetId: integer("beatmapset_id").references(() => beatmapset.id, {
      onDelete: "cascade",
    }),
    difficulty: varchar("difficulty", { length: 255 }).notNull(),
    osuHash: varchar("osu_hash", { length: 128 }).notNull().unique(),
    notesHash: varchar("notes_hash", { length: 128 }).notNull().unique(),
    countCircles: integer("count_circles").notNull(),
    countSliders: integer("count_sliders").notNull(),
    countSpinners: integer("count_spinners").notNull(),
    maxCombo: integer("max_combo").notNull(),
    mainPattern: jsonb("main_pattern").notNull(),
    drainTime: integer("drain_time").notNull(),
    totalTime: integer("total_time").notNull(),
    bpm: decimal("bpm", { precision: 10, scale: 2 }).notNull(),
    cs: decimal("cs", { precision: 3, scale: 1 }).notNull(),
    ar: decimal("ar", { precision: 3, scale: 1 }).notNull(),
    od: decimal("od", { precision: 3, scale: 1 }).notNull(),
    hp: decimal("hp", { precision: 3, scale: 1 }).notNull(),
    mode: smallint("mode").notNull().default(3),
    status: varchar("status", { length: 20 }).notNull().default("pending"),
    createdAt: timestamp("created_at").defaultNow(),
    updatedAt: timestamp("updated_at").defaultNow(),
  },
  (t) => [
    index("idx_beatmap_beatmapset_id").on(t.beatmapsetId),
    index("idx_beatmap_mode").on(t.mode),
    index("idx_beatmap_status").on(t.status),
    index("idx_beatmap_bpm").on(t.bpm),
    index("idx_beatmap_total_time").on(t.totalTime),
    index("idx_beatmap_od").on(t.od),
  ],
);

export const beatmapDuplicate = pgTable(
  "beatmap_duplicate",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    canonicalBeatmapId: integer("canonical_beatmap_id").notNull().references(
      () => beatmap.id,
      { onDelete: "cascade" },
    ),
    osuId: integer("osu_id").unique(),
    beatmapsetId: integer("beatmapset_id").references(() => beatmapset.id, {
      onDelete: "set null",
    }),
    difficulty: varchar("difficulty", { length: 255 }).notNull(),
    osuHash: varchar("osu_hash", { length: 128 }).notNull().unique(),
    notesHash: varchar("notes_hash", { length: 128 }).notNull(),
    countCircles: integer("count_circles").notNull(),
    countSliders: integer("count_sliders").notNull(),
    countSpinners: integer("count_spinners").notNull(),
    maxCombo: integer("max_combo").notNull(),
    mainPattern: jsonb("main_pattern").notNull(),
    drainTime: integer("drain_time").notNull(),
    totalTime: integer("total_time").notNull(),
    bpm: decimal("bpm", { precision: 10, scale: 2 }).notNull(),
    cs: decimal("cs", { precision: 3, scale: 1 }).notNull(),
    ar: decimal("ar", { precision: 3, scale: 1 }).notNull(),
    od: decimal("od", { precision: 3, scale: 1 }).notNull(),
    hp: decimal("hp", { precision: 3, scale: 1 }).notNull(),
    mode: smallint("mode").notNull().default(3),
    status: varchar("status", { length: 20 }).notNull().default("pending"),
    createdAt: timestamp("created_at").defaultNow(),
  },
  (t) => [
    index("idx_beatmap_dup_canonical").on(t.canonicalBeatmapId),
    index("idx_beatmap_dup_notes_hash").on(t.notesHash),
  ],
);

export const beatmapRating = pgTable(
  "beatmap_rating",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    beatmapId: integer("beatmap_id").notNull().references(() => beatmap.id, {
      onDelete: "cascade",
    }),
    rating: decimal("rating", { precision: 8, scale: 4 }).notNull(),
    ratingType: varchar("rating_type", { length: 30 }).notNull(),
    createdAt: timestamp("created_at").defaultNow(),
    updatedAt: timestamp("updated_at").defaultNow(),
  },
  (t) => [
    unique("uq_beatmap_rating_type").on(t.beatmapId, t.ratingType),
    index("idx_beatmap_rating_beatmap_id").on(t.beatmapId),
    index("idx_beatmap_rating_type_rating").on(t.ratingType, t.rating),
  ],
);

export const beatmapManiaSkill = pgTable(
  "beatmap_mania_skill",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    ratingId: integer("rating_id").notNull().unique().references(
      () => beatmapRating.id,
      { onDelete: "cascade" },
    ),
    stream: decimal("stream", { precision: 8, scale: 4 }),
    jumpstream: decimal("jumpstream", { precision: 8, scale: 4 }),
    handstream: decimal("handstream", { precision: 8, scale: 4 }),
    stamina: decimal("stamina", { precision: 8, scale: 4 }),
    jackspeed: decimal("jackspeed", { precision: 8, scale: 4 }),
    chordjack: decimal("chordjack", { precision: 8, scale: 4 }),
    technical: decimal("technical", { precision: 8, scale: 4 }),
  },
  (t) => [
    index("idx_mania_skill_stream").on(t.stream),
    index("idx_mania_skill_jumpstream").on(t.jumpstream),
    index("idx_mania_skill_handstream").on(t.handstream),
    index("idx_mania_skill_stamina").on(t.stamina),
    index("idx_mania_skill_jackspeed").on(t.jackspeed),
    index("idx_mania_skill_chordjack").on(t.chordjack),
    index("idx_mania_skill_technical").on(t.technical),
  ],
);

export const pendingBeatmap = pgTable(
  "pending_beatmap",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    osuHash: text("osu_hash").notNull().unique(),
    osuId: integer("osu_id"),
    createdAt: timestamp("created_at").defaultNow(),
  },
  (t) => [index("idx_pending_beatmap_created_at").on(t.createdAt)],
);

export const failedQuery = pgTable(
  "failed_query",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    hash: text("hash").notNull(),
    createdAt: timestamp("created_at").defaultNow(),
  },
  (t) => [
    index("idx_failed_query_hash").on(t.hash),
    index("idx_failed_query_created_at").on(t.createdAt),
  ],
);
