import { decimal, index, integer, pgTable } from "drizzle-orm/pg-core";
import { beatmapRating } from "./beatmap_rating.ts";

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
