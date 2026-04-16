import type { db as DB } from "@/db/client.ts";

export function getBeatmapset(db: typeof DB, opts: { id?: number | null; osuId?: number | null }) {
  if (opts.id != null) {
    return db.query.beatmapset.findFirst({ where: (bs, { eq }) => eq(bs.id, opts.id!) });
  }
  if (opts.osuId != null) {
    return db.query.beatmapset.findFirst({ where: (bs, { eq }) => eq(bs.osuId, opts.osuId!) });
  }
  return null;
}
