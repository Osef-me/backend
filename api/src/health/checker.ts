import { sql } from "drizzle-orm";
import { db } from "../db/client.ts";
import { serviceLog } from "../db/schemas/service_log.ts";
import { and, eq, isNull } from "drizzle-orm";

type ServiceName = "database" | "api" | "calculator";
type ServiceStatus = "operational" | "degraded" | "outage";

const INTERVAL_MS = 30_000;
const CALCULATOR_URL = Deno.env.get("CALCULATOR_URL") ?? "http://localhost:4000";
const API_URL        = Deno.env.get("API_SELF_URL")   ?? "http://localhost:3000";

// ── individual checks ────────────────────────────────────

async function checkDatabase(): Promise<ServiceStatus> {
  try {
    await db.execute(sql`SELECT 1`);
    return "operational";
  } catch {
    return "outage";
  }
}

async function checkApi(): Promise<ServiceStatus> {
  try {
    const res = await fetch(`${API_URL}/health`, { signal: AbortSignal.timeout(5000) });
    return res.ok ? "operational" : "degraded";
  } catch {
    return "outage";
  }
}

async function checkCalculator(): Promise<ServiceStatus> {
  try {
    const res = await fetch(`${CALCULATOR_URL}/health`, { signal: AbortSignal.timeout(5000) });
    return res.ok ? "operational" : "degraded";
  } catch {
    return "outage";
  }
}

// ── state tracking ───────────────────────────────────────

type ServiceState = { status: ServiceStatus; since: Date; logId: number };
const state = new Map<ServiceName, ServiceState>();

async function closeEntry(logId: number, now: Date, since: Date) {
  const duration = Math.floor((now.getTime() - since.getTime()) / 1000);
  await db
    .update(serviceLog)
    .set({ endedAt: now, duration })
    .where(eq(serviceLog.id, logId));
}

async function openEntry(service: ServiceName, status: ServiceStatus, now: Date): Promise<number> {
  const [row] = await db
    .insert(serviceLog)
    .values({ service, status, startedAt: now })
    .returning({ id: serviceLog.id });
  return row.id;
}

async function handleService(service: ServiceName, newStatus: ServiceStatus) {
  const now = new Date();
  const prev = state.get(service);

  if (!prev) {
    // first check — open initial entry
    const logId = await openEntry(service, newStatus, now);
    state.set(service, { status: newStatus, since: now, logId });
    console.log(`[health] ${service}: ${newStatus} (initial)`);
    return;
  }

  if (prev.status === newStatus) return; // no change

  // status changed — close previous, open new
  await closeEntry(prev.logId, now, prev.since);

  const comment = buildComment(service, prev.status, newStatus);
  const [row] = await db
    .insert(serviceLog)
    .values({ service, status: newStatus, startedAt: now, comment })
    .returning({ id: serviceLog.id });

  state.set(service, { status: newStatus, since: now, logId: row.id });
  console.log(`[health] ${service}: ${prev.status} → ${newStatus}`);
}

function buildComment(svc: ServiceName, from: ServiceStatus, to: ServiceStatus): string {
  if (to === "operational") {
    return `${label(svc)} recovered from ${from}. Service restored.`;
  }
  if (to === "outage") {
    return `${label(svc)} is unreachable. Incident opened.`;
  }
  return `${label(svc)} is experiencing degraded performance.`;
}

function label(svc: ServiceName) {
  return { database: "Database", api: "API", calculator: "Calculator" }[svc];
}

// ── main loop ────────────────────────────────────────────

async function runChecks() {
  const [dbStatus, apiStatus, calcStatus] = await Promise.all([
    checkDatabase(),
    checkApi(),
    checkCalculator(),
  ]);

  await handleService("database",   dbStatus);
  await handleService("api",        apiStatus);
  await handleService("calculator", calcStatus);
}

export function startHealthChecker() {
  // first run immediately, then every 30s
  runChecks().catch(console.error);
  setInterval(() => runChecks().catch(console.error), INTERVAL_MS);
  console.log(`[health] checker started (interval: ${INTERVAL_MS / 1000}s)`);
}
