import { integer, pgEnum, pgTable, text, timestamp } from "drizzle-orm/pg-core";

export const serviceNameEnum   = pgEnum("service_name",   ["database", "api", "calculator"]);
export const serviceStatusEnum = pgEnum("service_status", ["operational", "degraded", "outage"]);

export const serviceLog = pgTable("service_log", {
  id:        integer("id").generatedAlwaysAsIdentity().primaryKey(),
  service:   serviceNameEnum("service").notNull(),
  status:    serviceStatusEnum("status").notNull(),
  startedAt: timestamp("started_at").notNull().defaultNow(),
  endedAt:   timestamp("ended_at"),
  duration:  integer("duration"),
  comment:   text("comment"),
  createdAt: timestamp("created_at").notNull().defaultNow(),
});
