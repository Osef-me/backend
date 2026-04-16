import { drizzle } from "drizzle-orm/postgres-js";
import postgres from "postgres";
import { faker } from "npm:@faker-js/faker@^9";
import * as schema from "../db/schemas/index.ts";

const client = postgres(Deno.env.get("DATABASE_URL")!);
const db = drizzle(client, { schema });

// ── factories ─────────────────────────────────────────────

function beatmapsetFactory(osuId: number) {
  return {
    osuId,
    artist: faker.person.fullName(),
    artistUnicode: faker.helpers.maybe(() => faker.person.fullName()),
    title: faker.lorem.words({ min: 1, max: 4 }),
    titleUnicode: faker.helpers.maybe(() => faker.lorem.words({ min: 1, max: 4 })),
    creator: faker.internet.username(),
    source: faker.helpers.maybe(() => faker.lorem.word()),
    tags: faker.helpers.multiple(() => faker.lorem.word(), { count: { min: 0, max: 8 } }),
    hasVideo: faker.datatype.boolean({ probability: 0.2 }),
    hasStoryboard: faker.datatype.boolean({ probability: 0.15 }),
    isExplicit: faker.datatype.boolean({ probability: 0.05 }),
    isFeatured: faker.datatype.boolean({ probability: 0.1 }),
    coverUrl: `https://assets.ppy.sh/beatmaps/${osuId}/covers/cover.jpg`,
    previewUrl: `https://b.ppy.sh/preview/${osuId}.mp3`,
    osuFileUrl: null,
    osuStatusChangedAt: faker.date.past({ years: 3 }),
  };
}

const DIFFICULTY_NAMES = ["Easy", "Normal", "Hard", "Insane", "Extra", "Expert", "Beginner", "Advanced", "Lunatic"];
const STATUSES = ["pending", "ranked", "qualified", "loved", "graveyard"] as const;

function beatmapFactory(beatmapsetId: number, osuId: number) {
  const bpm = faker.number.float({ min: 80, max: 300, fractionDigits: 2 });
  const circles = faker.number.int({ min: 0, max: 2000 });
  const sliders = faker.number.int({ min: 0, max: 1500 });
  const spinners = faker.number.int({ min: 0, max: 20 });
  const drainTime = faker.number.int({ min: 60, max: 480 });

  return {
    osuId,
    beatmapsetId,
    difficulty: faker.helpers.arrayElement(DIFFICULTY_NAMES),
    osuHash: faker.string.hexadecimal({ length: 32, casing: "lower", prefix: "" }),
    notesHash: faker.string.hexadecimal({ length: 32, casing: "lower", prefix: "" }),
    countCircles: circles,
    countSliders: sliders,
    countSpinners: spinners,
    maxCombo: circles + sliders * 2 + spinners,
    mainPattern: {
      streams: faker.number.float({ min: 0, max: 1, fractionDigits: 2 }),
      jumps: faker.number.float({ min: 0, max: 1, fractionDigits: 2 }),
      tech: faker.number.float({ min: 0, max: 1, fractionDigits: 2 }),
    },
    drainTime,
    totalTime: drainTime + faker.number.int({ min: 0, max: 30 }),
    bpm: bpm.toString(),
    cs: faker.number.float({ min: 0, max: 7, fractionDigits: 1 }).toString(),
    ar: faker.number.float({ min: 0, max: 10, fractionDigits: 1 }).toString(),
    od: faker.number.float({ min: 0, max: 10, fractionDigits: 1 }).toString(),
    hp: faker.number.float({ min: 0, max: 10, fractionDigits: 1 }).toString(),
    mode: 3,
    status: faker.helpers.weightedArrayElement([
      { weight: 5, value: "ranked" },
      { weight: 2, value: "loved" },
      { weight: 1, value: "qualified" },
      { weight: 1, value: "graveyard" },
      { weight: 1, value: "pending" },
    ]),
  };
}

// ── seed ──────────────────────────────────────────────────

const BEATMAPSET_COUNT = Number(Deno.args[0] ?? 50);
const BEATMAPS_PER_SET = { min: 1, max: 8 };

console.log(`Seeding ${BEATMAPSET_COUNT} beatmapsets...`);

let osuBeatmapsetId = 1_000_000;
let osuBeatmapId = 5_000_000;

for (let i = 0; i < BEATMAPSET_COUNT; i++) {
  osuBeatmapsetId++;

  const [set] = await db
    .insert(schema.beatmapset)
    .values(beatmapsetFactory(osuBeatmapsetId))
    .returning({ id: schema.beatmapset.id });

  const beatmapCount = faker.number.int(BEATMAPS_PER_SET);
  const beatmaps = Array.from({ length: beatmapCount }, () => {
    osuBeatmapId++;
    return beatmapFactory(set.id, osuBeatmapId);
  });

  await db.insert(schema.beatmap).values(beatmaps);

  if ((i + 1) % 10 === 0) {
    console.log(`  ${i + 1}/${BEATMAPSET_COUNT} beatmapsets inserted`);
  }
}

console.log("Done.");
await client.end();
