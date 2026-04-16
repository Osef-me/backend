import { builder } from "../builder.ts";

export const OrderDirEnum = builder.enumType("OrderDir", {
  values: {
    ASC: { value: "asc" },
    DESC: { value: "desc" },
  } as const,
});

export const PaginationInput = builder.inputType("Pagination", {
  fields: (t) => ({
    limit: t.int({ required: false, defaultValue: 20 }),
    offset: t.int({ required: false, defaultValue: 0 }),
  }),
});

export type Pagination = {
  limit?: number | null;
  offset?: number | null;
};

export type OrderDir = "asc" | "desc";
