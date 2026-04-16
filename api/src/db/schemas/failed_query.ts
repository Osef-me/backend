import { index, integer, pgTable, text, timestamp } from "drizzle-orm/pg-core";

export const failedQuery = pgTable(
  "failed_query",
  {
    id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
    hash: text("hash").notNull(),
    createdAt: timestamp("created_at").defaultNow(),
  },
  (t) => [
    index("idx_failed_query_hash").on(t.hash),
    index("idx_failed_query_created_at").on(t.createdAt),
  ],
);
