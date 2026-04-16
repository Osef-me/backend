import { and, type SQL } from "drizzle-orm";

export function buildWhere(conditions: SQL[]): SQL | undefined {
  return conditions.length > 0 ? and(...conditions) : undefined;
}
