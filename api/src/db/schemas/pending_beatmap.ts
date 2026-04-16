import { index, integer, pgTable, text, timestamp } from "drizzle-orm/pg-core";

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
