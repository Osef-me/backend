import { asc, desc } from "drizzle-orm";
import { beatmap } from "@/db/schemas/beatmap.ts";
import type { BeatmapOrderBy } from "@/types/beatmap.ts";

const COLUMNS = {
  bpm: beatmap.bpm,
  od: beatmap.od,
  cs: beatmap.cs,
  ar: beatmap.ar,
  hp: beatmap.hp,
  totalTime: beatmap.totalTime,
  drainTime: beatmap.drainTime,
  maxCombo: beatmap.maxCombo,
  createdAt: beatmap.createdAt,
} as const;

function toOrderClause(field: keyof typeof COLUMNS, dir: "asc" | "desc") {
  return dir === "asc" ? asc(COLUMNS[field]) : desc(COLUMNS[field]);
}

export function buildOrderBy(orderBy: BeatmapOrderBy | null | undefined) {
  if (!orderBy) return [desc(beatmap.createdAt)];
  return (Object.entries(orderBy) as [keyof typeof COLUMNS, "asc" | "desc"][])
    .filter(([, dir]) => dir != null)
    .map(([field, dir]) => toOrderClause(field, dir));
}
