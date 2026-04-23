import { sql } from "drizzle-orm";
import { db } from "../db/client.ts";

export const SERVER_STARTED_AT = Date.now();

export const counters = {
  requests: 0,
};

export async function getServerLoad() {
  const mem = Deno.memoryUsage();
  const rows = await db.execute(sql`
    SELECT
      (count(*) FILTER (WHERE datname = current_database()))::int AS total,
      (count(*) FILTER (WHERE datname = current_database() AND state = 'active'))::int AS active,
      (count(*) FILTER (WHERE datname = current_database() AND state = 'idle'))::int AS idle
    FROM pg_stat_activity
  `);
  const [row] = rows as unknown as { total: number; active: number; idle: number }[];

  return {
    uptimeSec: Math.floor((Date.now() - SERVER_STARTED_AT) / 1000),
    memory: {
      rssBytes: mem.rss,
      heapUsedBytes: mem.heapUsed,
      heapTotalBytes: mem.heapTotal,
      externalBytes: mem.external,
    },
    requests: counters.requests,
    db: {
      total: Number(row?.total ?? 0),
      active: Number(row?.active ?? 0),
      idle: Number(row?.idle ?? 0),
    },
    now: new Date().toISOString(),
  };
}
