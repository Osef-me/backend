import { integer, pgTable, text, timestamp } from "drizzle-orm/pg-core";

export const seederState = pgTable("seeder_state", {
  id: integer("id").generatedAlwaysAsIdentity().primaryKey(),
  key: text("key").notNull().unique(),
  value: text("value").notNull(),
  updatedAt: timestamp("updated_at").defaultNow(),
});
