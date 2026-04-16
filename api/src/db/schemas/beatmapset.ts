import {
  boolean,
  index,
  integer,
  pgTable,
  text,
  timestamp,
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
