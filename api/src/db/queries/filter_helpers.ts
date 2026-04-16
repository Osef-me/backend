import { and, eq, gt, gte, ilike, inArray, lt, lte, type SQL } from "drizzle-orm";
import type { AnyColumn } from "drizzle-orm";
import type { BoolFilter, FloatFilter, IntFilter, StringFilter } from "../../schema/inputs/filters.ts";

export function applyIntFilter(col: AnyColumn, f: IntFilter | null | undefined): SQL[] {
  if (!f) return [];
  const r: SQL[] = [];
  if (f.eq != null) r.push(eq(col, f.eq));
  if (f.gt != null) r.push(gt(col, f.gt));
  if (f.gte != null) r.push(gte(col, f.gte));
  if (f.lt != null) r.push(lt(col, f.lt));
  if (f.lte != null) r.push(lte(col, f.lte));
  if (f.in?.length) r.push(inArray(col, f.in));
  return r;
}

export function applyFloatFilter(col: AnyColumn, f: FloatFilter | null | undefined): SQL[] {
  if (!f) return [];
  const r: SQL[] = [];
  if (f.eq != null) r.push(eq(col, f.eq));
  if (f.gt != null) r.push(gt(col, f.gt));
  if (f.gte != null) r.push(gte(col, f.gte));
  if (f.lt != null) r.push(lt(col, f.lt));
  if (f.lte != null) r.push(lte(col, f.lte));
  return r;
}

export function applyStringFilter(col: AnyColumn, f: StringFilter | null | undefined): SQL[] {
  if (!f) return [];
  const r: SQL[] = [];
  if (f.eq != null) r.push(eq(col, f.eq));
  if (f.contains != null) r.push(ilike(col, `%${f.contains}%`));
  if (f.in?.length) r.push(inArray(col, f.in));
  return r;
}

export function applyBoolFilter(col: AnyColumn, f: BoolFilter | null | undefined): SQL[] {
  if (!f || f.eq == null) return [];
  return [eq(col, f.eq)];
}

export function buildWhere(conditions: SQL[]): SQL | undefined {
  return conditions.length > 0 ? and(...conditions) : undefined;
}
