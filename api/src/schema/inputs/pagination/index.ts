import { builder } from "@/schema/builder.ts";

export const OrderDirEnum = builder.enumType("OrderDir", {
  values: { ASC: { value: "asc" }, DESC: { value: "desc" } } as const,
});

export const PaginationInput = builder.inputType("Pagination", {
  fields: (t) => ({
    limit: t.int({ required: false, defaultValue: 20 }),
    offset: t.int({ required: false, defaultValue: 0 }),
  }),
});
