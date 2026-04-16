import { eq, type SQL } from "drizzle-orm";
import type { AnyColumn } from "drizzle-orm";
import type { BoolFilter } from "@/types/filters.ts";

export function applyBoolFilter(col: AnyColumn, f: BoolFilter | null | undefined): SQL[] {
  if (!f || f.eq == null) return [];
  return [eq(col, f.eq)];
}
