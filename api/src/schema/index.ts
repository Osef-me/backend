import { builder } from "./builder.ts";

// Object types
import "./types/beatmapset.ts";
import "./types/beatmap_mania_skill.ts";
import "./types/beatmap_mania_ratio.ts";
import "./types/beatmap_rating.ts";
import "./types/beatmap.ts";
import "./types/beatmap_duplicate.ts";

// Queries
import "./queries/beatmap/index.ts";
import "./queries/beatmapset/index.ts";

export const schema = builder.toSchema();
