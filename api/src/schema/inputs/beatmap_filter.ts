import { builder } from "../builder.ts";
import {
  FloatFilterInput,
  IntFilterInput,
  StringFilterInput,
} from "./filters.ts";
import type { FloatFilter, IntFilter, StringFilter } from "./filters.ts";
import { BeatmapsetFilterInput } from "./beatmapset_filter.ts";
import type { BeatmapsetFilter } from "./beatmapset_filter.ts";
import { OrderDirEnum } from "./pagination.ts";
import type { OrderDir } from "./pagination.ts";

const ManiaSkillFilterInput = builder.inputType("ManiaSkillFilter", {
  fields: (t) => ({
    stream: t.field({ type: FloatFilterInput, required: false }),
    jumpstream: t.field({ type: FloatFilterInput, required: false }),
    handstream: t.field({ type: FloatFilterInput, required: false }),
    stamina: t.field({ type: FloatFilterInput, required: false }),
    jackspeed: t.field({ type: FloatFilterInput, required: false }),
    chordjack: t.field({ type: FloatFilterInput, required: false }),
    technical: t.field({ type: FloatFilterInput, required: false }),
  }),
});

const ManiaRatioFilterInput = builder.inputType("ManiaRatioFilter", {
  fields: (t) => ({
    stream: t.field({ type: FloatFilterInput, required: false }),
    jumpstream: t.field({ type: FloatFilterInput, required: false }),
    handstream: t.field({ type: FloatFilterInput, required: false }),
    stamina: t.field({ type: FloatFilterInput, required: false }),
    jackspeed: t.field({ type: FloatFilterInput, required: false }),
    chordjack: t.field({ type: FloatFilterInput, required: false }),
    technical: t.field({ type: FloatFilterInput, required: false }),
  }),
});

// Rating filter — type is required so we know which rating system to join
const RatingFilterInput = builder.inputType("RatingFilter", {
  fields: (t) => ({
    type: t.string({ required: true }), // e.g. "osu2016", "osu_current", "etterna", "quaver", "interlude", "sunnyxxy"
    value: t.field({ type: FloatFilterInput, required: false }),
    skill: t.field({ type: ManiaSkillFilterInput, required: false }),
  }),
});

export const BeatmapFilterInput = builder.inputType("BeatmapFilter", {
  fields: (t) => ({
    id: t.field({ type: IntFilterInput, required: false }),
    osuId: t.field({ type: IntFilterInput, required: false }),
    mode: t.field({ type: IntFilterInput, required: false }),
    status: t.field({ type: StringFilterInput, required: false }),
    bpm: t.field({ type: FloatFilterInput, required: false }),
    cs: t.field({ type: FloatFilterInput, required: false }),
    ar: t.field({ type: FloatFilterInput, required: false }),
    od: t.field({ type: FloatFilterInput, required: false }),
    hp: t.field({ type: FloatFilterInput, required: false }),
    totalTime: t.field({ type: IntFilterInput, required: false }),
    drainTime: t.field({ type: IntFilterInput, required: false }),
    maxCombo: t.field({ type: IntFilterInput, required: false }),
    countCircles: t.field({ type: IntFilterInput, required: false }),
    countSliders: t.field({ type: IntFilterInput, required: false }),
    countSpinners: t.field({ type: IntFilterInput, required: false }),
    beatmapset: t.field({ type: BeatmapsetFilterInput, required: false }),
    ratio: t.field({ type: ManiaRatioFilterInput, required: false }),
    rating: t.field({ type: RatingFilterInput, required: false }),
  }),
});

export const BeatmapOrderByInput = builder.inputType("BeatmapOrderBy", {
  fields: (t) => ({
    bpm: t.field({ type: OrderDirEnum, required: false }),
    od: t.field({ type: OrderDirEnum, required: false }),
    cs: t.field({ type: OrderDirEnum, required: false }),
    ar: t.field({ type: OrderDirEnum, required: false }),
    hp: t.field({ type: OrderDirEnum, required: false }),
    totalTime: t.field({ type: OrderDirEnum, required: false }),
    drainTime: t.field({ type: OrderDirEnum, required: false }),
    maxCombo: t.field({ type: OrderDirEnum, required: false }),
    createdAt: t.field({ type: OrderDirEnum, required: false }),
  }),
});

export type ManiaSkillFilter = {
  stream?: FloatFilter | null;
  jumpstream?: FloatFilter | null;
  handstream?: FloatFilter | null;
  stamina?: FloatFilter | null;
  jackspeed?: FloatFilter | null;
  chordjack?: FloatFilter | null;
  technical?: FloatFilter | null;
};

export type ManiaRatioFilter = ManiaSkillFilter;

export type RatingFilter = {
  type: string;
  value?: FloatFilter | null;
  skill?: ManiaSkillFilter | null;
};

export type BeatmapFilter = {
  id?: IntFilter | null;
  osuId?: IntFilter | null;
  mode?: IntFilter | null;
  status?: StringFilter | null;
  bpm?: FloatFilter | null;
  cs?: FloatFilter | null;
  ar?: FloatFilter | null;
  od?: FloatFilter | null;
  hp?: FloatFilter | null;
  totalTime?: IntFilter | null;
  drainTime?: IntFilter | null;
  maxCombo?: IntFilter | null;
  countCircles?: IntFilter | null;
  countSliders?: IntFilter | null;
  countSpinners?: IntFilter | null;
  beatmapset?: BeatmapsetFilter | null;
  ratio?: ManiaRatioFilter | null;
  rating?: RatingFilter | null;
};

export type BeatmapOrderBy = {
  bpm?: OrderDir | null;
  od?: OrderDir | null;
  cs?: OrderDir | null;
  ar?: OrderDir | null;
  hp?: OrderDir | null;
  totalTime?: OrderDir | null;
  drainTime?: OrderDir | null;
  maxCombo?: OrderDir | null;
  createdAt?: OrderDir | null;
};
