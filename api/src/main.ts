import { Hono } from "hono";
import { logger } from "hono/logger";
import { createYoga } from "graphql-yoga";
import { desc, gte, sql } from "drizzle-orm";
import { startHealthChecker } from "./health/checker.ts";
import { db } from "./db/client.ts";
import { schema } from "./schema/index.ts";
import { serviceLog } from "./db/schemas/service_log.ts";
import { processorRouter } from "./processor/route.ts";
import { pendingRouter } from "./internal/pending.ts";
import { requireInternalAuth } from "./middleware/auth.ts";
import { rateLimit } from "./middleware/rate_limit.ts";
import { getStats } from "./stats/stats.ts";

const landing = await Deno.readTextFile(new URL("./landing.html", import.meta.url));
const statusPage = await Deno.readTextFile(new URL("./status.html", import.meta.url));
const statsPage = await Deno.readTextFile(new URL("./stats.html", import.meta.url));

const app = new Hono();

app.use("*", logger());

const yoga = createYoga({
  schema,
  context: () => ({ db }),
  graphiql: Deno.env.get("NODE_ENV") !== "production",
});

app.use("/graphql", (c) => yoga.handle(c.req.raw, c));

app.get("/", (c) => c.html(landing));
app.get("/status", (c) => c.html(statusPage));
app.get("/stats", (c) => c.html(statsPage));

let statsCache: { at: number; data: unknown } | null = null;
app.get("/api/stats", async (c) => {
  const now = Date.now();
  if (statsCache && now - statsCache.at < 60_000) {
    return c.json(statsCache.data);
  }
  const data = await getStats();
  statsCache = { at: now, data };
  return c.json(data);
});

app.get("/api/status", async (c) => {
  const since = new Date();
  since.setDate(since.getDate() - 60);
  since.setHours(0, 0, 0, 0);

  const rows = await db
    .select()
    .from(serviceLog)
    .where(gte(serviceLog.startedAt, since))
    .orderBy(desc(serviceLog.startedAt));

  const SERVICES = ["database", "api", "calculator"] as const;
  const today = new Date();

  const data = SERVICES.map((svc) => {
    const svcRows = rows.filter((r) => r.service === svc);

    const days = Array.from({ length: 60 }, (_, i) => {
      const d = new Date(today);
      d.setDate(d.getDate() - (59 - i));
      const dateStr = d.toISOString().slice(0, 10);

      const dayRows = svcRows.filter((r) => {
        const s = r.startedAt.toISOString().slice(0, 10);
        const e = r.endedAt ? r.endedAt.toISOString().slice(0, 10) : s;
        return s <= dateStr && e >= dateStr;
      });

      if (dayRows.length === 0) return { date: dateStr, status: "no_data" };
      if (dayRows.some((r) => r.status === "outage"))     return { date: dateStr, status: "outage" };
      if (dayRows.some((r) => r.status === "degraded"))   return { date: dateStr, status: "degraded" };
      return { date: dateStr, status: "operational" };
    });

    // current status = most recent row
    const current = svcRows[0]?.status ?? "operational";

    // incidents (non-operational rows with comments)
    const incidents = svcRows
      .filter((r) => r.status !== "operational" && r.comment)
      .slice(0, 10)
      .map((r) => ({
        status:    r.status,
        startedAt: r.startedAt.toISOString(),
        endedAt:   r.endedAt?.toISOString() ?? null,
        duration:  r.duration,
        comment:   r.comment,
      }));

    return { service: svc, current, days, incidents };
  });

  return c.json(data);
});

app.use("/api/*", rateLimit({ windowMs: 60_000, max: 100 }));
app.use("/api/internal/*", requireInternalAuth);

app.route("/api/beatmap", processorRouter);
app.route("/api/internal/pending", pendingRouter);

app.get("/health", (c) => c.json({ status: "ok" }));

startHealthChecker();

const port = Number(Deno.env.get("PORT") ?? 3000);
Deno.serve({ port }, app.fetch);
