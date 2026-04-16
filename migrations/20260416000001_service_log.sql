CREATE TYPE service_name   AS ENUM ('database', 'api', 'calculator');
CREATE TYPE service_status AS ENUM ('operational', 'degraded', 'outage');

CREATE TABLE service_log (
    id         INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    service    service_name   NOT NULL,
    status     service_status NOT NULL,
    started_at TIMESTAMP      NOT NULL DEFAULT NOW(),
    ended_at   TIMESTAMP,
    duration   INTEGER,                         -- seconds, null if still ongoing
    comment    TEXT,
    created_at TIMESTAMP      NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_service_log_service    ON service_log(service);
CREATE INDEX idx_service_log_started_at ON service_log(started_at DESC);
