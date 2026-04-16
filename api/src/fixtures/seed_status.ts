import { drizzle } from "drizzle-orm/postgres-js";
import postgres from "postgres";
import { faker } from "npm:@faker-js/faker@^9";
import * as schema from "../db/schemas/index.ts";

const client = postgres(Deno.env.get("DATABASE_URL")!);
const db = drizzle(client, { schema });

const SERVICES = ["database", "api", "calculator"] as const;
const NOW = new Date();

const rows: (typeof schema.serviceLog.$inferInsert)[] = [];

for (const service of SERVICES) {
  for (let d = 59; d >= 0; d--) {
    const dayStart = new Date(NOW);
    dayStart.setDate(dayStart.getDate() - d);
    dayStart.setHours(0, 0, 0, 0);

    // ~85% chance operational day, ~10% degraded, ~5% outage
    const roll = Math.random();
    const hasIncident = roll > 0.82;

    if (!hasIncident) {
      // one clean "operational" entry covering the day
      rows.push({
        service,
        status: "operational",
        startedAt: dayStart,
        endedAt: new Date(dayStart.getTime() + 86_400_000),
        duration: 86400,
        comment: null,
      });
    } else {
      // outage or degraded window within the day
      const incidentStatus = Math.random() > 0.4 ? "degraded" : "outage";
      const offsetMs  = faker.number.int({ min: 1, max: 20 }) * 3_600_000;
      const durationS = faker.number.int({ min: 300, max: 7200 });
      const startedAt = new Date(dayStart.getTime() + offsetMs);
      const endedAt   = new Date(startedAt.getTime() + durationS * 1000);

      // operational before incident
      rows.push({
        service,
        status: "operational",
        startedAt: dayStart,
        endedAt: startedAt,
        duration: Math.floor(offsetMs / 1000),
        comment: null,
      });

      // the incident
      rows.push({
        service,
        status: incidentStatus,
        startedAt,
        endedAt,
        duration: durationS,
        comment: faker.helpers.arrayElement([
          "Elevated error rates detected, root cause identified and patched.",
          "Latency spike due to connection pool exhaustion.",
          "Unexpected restart after OOM event. Monitoring increased.",
          "Partial unavailability during deploy rollout.",
          "Intermittent failures — third-party dependency issue.",
          "Database replication lag exceeded threshold.",
          "Disk I/O saturation on primary node.",
        ]),
      });

      // back to operational after
      rows.push({
        service,
        status: "operational",
        startedAt: endedAt,
        endedAt: new Date(dayStart.getTime() + 86_400_000),
        duration: Math.floor((86_400_000 - offsetMs - durationS * 1000) / 1000),
        comment: null,
      });
    }
  }
}

await db.insert(schema.serviceLog).values(rows);
console.log(`Inserted ${rows.length} service_log rows.`);
await client.end();
