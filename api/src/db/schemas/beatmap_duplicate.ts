import { index, integer, pgTable, timestamp, varchar } from "drizzle-orm/pg-core";
import { beatmap } from "./beatmap.ts";

export const beatmapDuplicate = pgTable(
  "beatmap_duplicate",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    canonicalBeatmapId: integer("canonical_beatmap_id").notNull().references(
      () => beatmap.id,
      { onDelete: "cascade" },
    ),
    osuId: integer("osu_id").unique(),
    osuHash: varchar("osu_hash", { length: 128 }).notNull().unique(),
    notesHash: varchar("notes_hash", { length: 128 }).notNull(),
    createdAt: timestamp("created_at").defaultNow(),
  },
  (t) => [
    index("idx_beatmap_dup_canonical").on(t.canonicalBeatmapId),
    index("idx_beatmap_dup_notes_hash").on(t.notesHash),
  ],
);
