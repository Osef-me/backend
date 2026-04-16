FROM denoland/deno:alpine

WORKDIR /app

# Cache dependencies first (layer cache)
COPY api/deno.json .
RUN deno install

# Copy source
COPY api/src/ src/

RUN deno cache src/main.ts

CMD ["run", "--allow-net", "--allow-env", "--allow-read", "src/main.ts"]
