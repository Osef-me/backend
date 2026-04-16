import { BeatmapManiaRatioRef } from "./refs.ts";

BeatmapManiaRatioRef.implement({
  fields: (t) => ({
    id: t.exposeInt("id"),
    beatmapId: t.exposeInt("beatmapId"),
    stream: t.float({ nullable: true, resolve: (r) => r.stream ? parseFloat(r.stream) : null }),
    jumpstream: t.float({ nullable: true, resolve: (r) => r.jumpstream ? parseFloat(r.jumpstream) : null }),
    handstream: t.float({ nullable: true, resolve: (r) => r.handstream ? parseFloat(r.handstream) : null }),
    stamina: t.float({ nullable: true, resolve: (r) => r.stamina ? parseFloat(r.stamina) : null }),
    jackspeed: t.float({ nullable: true, resolve: (r) => r.jackspeed ? parseFloat(r.jackspeed) : null }),
    chordjack: t.float({ nullable: true, resolve: (r) => r.chordjack ? parseFloat(r.chordjack) : null }),
    technical: t.float({ nullable: true, resolve: (r) => r.technical ? parseFloat(r.technical) : null }),
  }),
});
