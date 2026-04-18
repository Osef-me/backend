import { Hono } from "hono";
import { db } from "../db/client.ts";
import { beatmap } from "../db/schemas/beatmap.ts";
import { beatmapRating } from "../db/schemas/beatmap_rating.ts";
import { beatmapManiaSkill } from "../db/schemas/beatmap_mania_skill.ts";
import { beatmapManiaRatio } from "../db/schemas/beatmap_mania_ratio.ts";
import { pendingBeatmap } from "../db/schemas/pending_beatmap.ts";
import { eq } from "drizzle-orm";
import { calculateRating } from "./client.ts";
import type { CalcType, RateResult } from "./types.ts";
import { isValidHash } from "../middleware/validate.ts";

export const processorRouter = new Hono();

const CALC_TYPE_TO_RATING_TYPE: Record<CalcType, string> = {
  osu2016: "osu2016",
  osu2018: "osu2018",
  osu_current: "osu_current",
  quaver2025: "quaver",
  interlude2025: "interlude",
  sunnyxxy: "sunnyxxy",
  etterna: "etterna",
};

const VALID_CALC_TYPES: CalcType[] = [
  "osu2016",
  "osu2018",
  "osu_current",
  "quaver2025",
  "interlude2025",
  "sunnyxxy",
  "etterna",
];

processorRouter.post("/calculate", async (c) => {
  const body = await c.req.json();
  const { calcType, centirates, hash, file } = body;

  if (!VALID_CALC_TYPES.includes(calcType)) {
    return c.json({ error: `unknown calc_type: ${calcType}` }, 400);
  }
  if (!Array.isArray(centirates) || centirates.length === 0) {
    return c.json({ error: "centirates must be a non-empty array" }, 400);
  }
  if (!hash && !file) {
    return c.json({ error: "provide hash or file" }, 400);
  }
  if (hash && !isValidHash(hash)) {
    return c.json({ error: "invalid hash format" }, 400);
  }

  const req = hash
    ? { calcType, centirates, normalizedHash: hash }
    : { calcType, centirates, file };

  let response;
  try {
    response = await calculateRating(req);
  } catch (e) {
    const err = e as Error;
    if (err.message.includes("not found")) {
      return c.json({ error: err.message }, 404);
    }
    return c.json({ error: err.message }, 500);
  }

  // Persist to DB if hash provided and centirate=100 present
  if (hash) {
    const result100 = response.results.find((r: RateResult) => r.centirate === 100);
    if (result100) {
      const beatmapRow = await db
        .select({ id: beatmap.id })
        .from(beatmap)
        .where(eq(beatmap.notesHash, response.normalizedHash))
        .then((rows) => rows[0] ?? null);

      if (beatmapRow) {
        await persistRating(beatmapRow.id, calcType, result100);
        await removeFromPending(response.normalizedHash);
      }
    }
  }

  return c.json(response);
});

async function persistRating(
  beatmapId: number,
  calcType: CalcType,
  result: RateResult,
) {
  const [ratingRow] = await db
    .insert(beatmapRating)
    .values({
      beatmapId,
      rating: String(result.rating),
      ratingType: CALC_TYPE_TO_RATING_TYPE[calcType],
    })
    .onConflictDoUpdate({
      target: [beatmapRating.beatmapId, beatmapRating.ratingType],
      set: { rating: String(result.rating), updatedAt: new Date() },
    })
    .returning({ id: beatmapRating.id });

  const skill = result.maniaSkill;
  await db
    .insert(beatmapManiaSkill)
    .values({
      ratingId: ratingRow.id,
      stream: String(skill.stream),
      jumpstream: String(skill.jumpstream),
      handstream: String(skill.handstream),
      stamina: String(skill.stamina),
      jackspeed: String(skill.jackspeed),
      chordjack: String(skill.chordjack),
      technical: String(skill.technical),
    })
    .onConflictDoUpdate({
      target: [beatmapManiaSkill.ratingId],
      set: {
        stream: String(skill.stream),
        jumpstream: String(skill.jumpstream),
        handstream: String(skill.handstream),
        stamina: String(skill.stamina),
        jackspeed: String(skill.jackspeed),
        chordjack: String(skill.chordjack),
        technical: String(skill.technical),
      },
    });

  if (result.rating > 0) {
    const ratio = {
      stream: String(skill.stream / result.rating),
      jumpstream: String(skill.jumpstream / result.rating),
      handstream: String(skill.handstream / result.rating),
      stamina: String(skill.stamina / result.rating),
      jackspeed: String(skill.jackspeed / result.rating),
      chordjack: String(skill.chordjack / result.rating),
      technical: String(skill.technical / result.rating),
    };
    await db
      .insert(beatmapManiaRatio)
      .values({ beatmapId, ...ratio })
      .onConflictDoUpdate({
        target: [beatmapManiaRatio.beatmapId],
        set: ratio,
      });
  }
}

async function removeFromPending(notesHash: string) {
  const beatmapRow = await db
    .select({ osuHash: beatmap.osuHash })
    .from(beatmap)
    .where(eq(beatmap.notesHash, notesHash))
    .then((rows) => rows[0] ?? null);

  if (beatmapRow) {
    await db.delete(pendingBeatmap).where(eq(pendingBeatmap.osuHash, beatmapRow.osuHash));
  }
}
