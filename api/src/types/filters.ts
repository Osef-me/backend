export type IntFilter = {
  eq?: number | null;
  gt?: number | null;
  gte?: number | null;
  lt?: number | null;
  lte?: number | null;
  in?: number[] | null;
};

export type FloatFilter = {
  eq?: number | null;
  gt?: number | null;
  gte?: number | null;
  lt?: number | null;
  lte?: number | null;
};

export type StringFilter = {
  eq?: string | null;
  contains?: string | null;
  in?: string[] | null;
};

export type BoolFilter = {
  eq?: boolean | null;
};
