import { eq, ilike, inArray, type SQL } from "drizzle-orm";
import type { AnyColumn } from "drizzle-orm";
import type { StringFilter } from "@/types/filters.ts";

export function applyStringFilter(col: AnyColumn, f: StringFilter | null | undefined): SQL[] {
  if (!f) return [];
  return [
    f.eq != null ? eq(col, f.eq) : undefined,
    f.contains != null ? ilike(col, `%${f.contains}%`) : undefined,
    f.in?.length ? inArray(col, f.in) : undefined,
  ].filter((x): x is SQL => x != null);
}
