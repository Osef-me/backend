import type { CalcRequest, CalcResponse } from "./types.ts";

const PROCESSOR_URL = Deno.env.get("PROCESSOR_URL") ?? "http://localhost:4000";

export async function calculateRating(req: CalcRequest): Promise<CalcResponse> {
  const res = await fetch(`${PROCESSOR_URL}/processor.Processor/Calculate`, {
    method: "POST",
    headers: { "Content-Type": "application/connect+json" },
    body: JSON.stringify(req),
    signal: AbortSignal.timeout(30_000),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: "unknown error" }));
    throw new Error(`processor error: ${err.message ?? res.statusText}`);
  }

  return res.json() as Promise<CalcResponse>;
}
