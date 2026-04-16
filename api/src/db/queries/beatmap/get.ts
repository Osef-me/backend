import type { db as DB } from "@/db/client.ts";

export function getBeatmap(db: typeof DB, opts: { id?: number | null; osuId?: number | null }) {
  if (opts.id != null) {
    return db.query.beatmap.findFirst({ where: (b, { eq }) => eq(b.id, opts.id!) });
  }
  if (opts.osuId != null) {
    return db.query.beatmap.findFirst({ where: (b, { eq }) => eq(b.osuId, opts.osuId!) });
  }
  return null;
}
