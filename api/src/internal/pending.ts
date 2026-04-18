import { Hono } from "hono";
import { db } from "../db/client.ts";
import { pendingBeatmap } from "../db/schemas/pending_beatmap.ts";
import { eq, asc, count } from "drizzle-orm";
import { isValidMd5Hash, sanitizePagination } from "../middleware/validate.ts";

export const pendingRouter = new Hono();

pendingRouter.get("/", async (c) => {
  const { limit, offset } = sanitizePagination(
    c.req.query("limit"),
    c.req.query("offset"),
  );

  const rows = await db
    .select({
      id: pendingBeatmap.id,
      osuHash: pendingBeatmap.osuHash,
      osuId: pendingBeatmap.osuId,
      createdAt: pendingBeatmap.createdAt,
    })
    .from(pendingBeatmap)
    .orderBy(asc(pendingBeatmap.createdAt))
    .limit(limit)
    .offset(offset);

  const [{ total }] = await db
    .select({ total: count() })
    .from(pendingBeatmap);

  return c.json({
    items: rows,
    total,
    limit,
    offset,
  });
});

pendingRouter.delete("/:osuHash", async (c) => {
  const osuHash = c.req.param("osuHash");

  if (!isValidMd5Hash(osuHash)) {
    return c.json({ error: "invalid hash format" }, 400);
  }

  const result = await db
    .delete(pendingBeatmap)
    .where(eq(pendingBeatmap.osuHash, osuHash))
    .returning({ id: pendingBeatmap.id });

  if (result.length === 0) {
    return c.json({ error: "not found" }, 404);
  }

  return c.json({ deleted: true, id: result[0].id });
});
