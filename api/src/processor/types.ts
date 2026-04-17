export type CalcType =
  | "osu2016"
  | "osu2018"
  | "osu_current"
  | "quaver2025"
  | "interlude2025"
  | "sunnyxxy"
  | "etterna";

export interface ManiaSkill {
  stream: number;
  jumpstream: number;
  handstream: number;
  stamina: number;
  jackspeed: number;
  chordjack: number;
  technical: number;
}

export interface RateResult {
  centirate: number;
  rating: number;
  maniaSkill: ManiaSkill;
}

export interface CalcResponse {
  normalizedHash: string;
  results: RateResult[];
}

export interface CalcRequestHash {
  calcType: CalcType;
  centirates: number[];
  normalizedHash: string;
}

export interface CalcRequestFile {
  calcType: CalcType;
  centirates: number[];
  file: { extension: string; content: string };
}

export type CalcRequest = CalcRequestHash | CalcRequestFile;
