import { BeatmapManiaSkillRef } from "./refs.ts";

BeatmapManiaSkillRef.implement({
  fields: (t) => ({
    id: t.exposeInt("id"),
    ratingId: t.exposeInt("ratingId"),
    stream: t.float({ nullable: true, resolve: (s) => s.stream ? parseFloat(s.stream) : null }),
    jumpstream: t.float({ nullable: true, resolve: (s) => s.jumpstream ? parseFloat(s.jumpstream) : null }),
    handstream: t.float({ nullable: true, resolve: (s) => s.handstream ? parseFloat(s.handstream) : null }),
    stamina: t.float({ nullable: true, resolve: (s) => s.stamina ? parseFloat(s.stamina) : null }),
    jackspeed: t.float({ nullable: true, resolve: (s) => s.jackspeed ? parseFloat(s.jackspeed) : null }),
    chordjack: t.float({ nullable: true, resolve: (s) => s.chordjack ? parseFloat(s.chordjack) : null }),
    technical: t.float({ nullable: true, resolve: (s) => s.technical ? parseFloat(s.technical) : null }),
  }),
});
