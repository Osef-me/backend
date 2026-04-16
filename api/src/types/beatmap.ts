import type { FloatFilter, IntFilter, StringFilter } from "./filters.ts";
import type { BeatmapsetFilter } from "./beatmapset.ts";
import type { OrderDir } from "./pagination.ts";

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
