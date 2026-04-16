import {
  decimal,
  index,
  integer,
  pgTable,
  timestamp,
  unique,
  varchar,
} from "drizzle-orm/pg-core";
import { beatmap } from "./beatmap.ts";

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
