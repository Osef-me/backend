import { decimal, index, integer, pgTable } from "drizzle-orm/pg-core";
import { beatmap } from "./beatmap.ts";

export const beatmapManiaRatio = pgTable(
  "beatmap_mania_ratio",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    beatmapId: integer("beatmap_id").notNull().unique().references(
      () => beatmap.id,
      { onDelete: "cascade" },
    ),
    stream: decimal("stream", { precision: 5, scale: 4 }),
    jumpstream: decimal("jumpstream", { precision: 5, scale: 4 }),
    handstream: decimal("handstream", { precision: 5, scale: 4 }),
    stamina: decimal("stamina", { precision: 5, scale: 4 }),
    jackspeed: decimal("jackspeed", { precision: 5, scale: 4 }),
    chordjack: decimal("chordjack", { precision: 5, scale: 4 }),
    technical: decimal("technical", { precision: 5, scale: 4 }),
  },
  (t) => [
    index("idx_mania_ratio_stream").on(t.stream),
    index("idx_mania_ratio_jumpstream").on(t.jumpstream),
    index("idx_mania_ratio_handstream").on(t.handstream),
    index("idx_mania_ratio_stamina").on(t.stamina),
    index("idx_mania_ratio_jackspeed").on(t.jackspeed),
    index("idx_mania_ratio_chordjack").on(t.chordjack),
    index("idx_mania_ratio_technical").on(t.technical),
  ],
);
