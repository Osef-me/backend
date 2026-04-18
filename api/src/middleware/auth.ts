import { Context, Next } from "hono";

const INTERNAL_API_KEY = Deno.env.get("INTERNAL_API_KEY");

export async function requireInternalAuth(c: Context, next: Next) {
  if (!INTERNAL_API_KEY) {
    console.warn("INTERNAL_API_KEY not set - internal routes unprotected");
    return next();
  }

  const providedKey = c.req.header("X-Internal-Key");
  if (providedKey !== INTERNAL_API_KEY) {
    return c.json({ error: "unauthorized" }, 401);
  }

  await next();
}
