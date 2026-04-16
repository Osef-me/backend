import { eq, gt, gte, lt, lte, type SQL } from "drizzle-orm";
import type { AnyColumn } from "drizzle-orm";
import type { FloatFilter } from "@/types/filters.ts";

export function applyFloatFilter(col: AnyColumn, f: FloatFilter | null | undefined): SQL[] {
  if (!f) return [];
  return [
    f.eq != null ? eq(col, f.eq) : undefined,
    f.gt != null ? gt(col, f.gt) : undefined,
    f.gte != null ? gte(col, f.gte) : undefined,
    f.lt != null ? lt(col, f.lt) : undefined,
    f.lte != null ? lte(col, f.lte) : undefined,
  ].filter((x): x is SQL => x != null);
}
