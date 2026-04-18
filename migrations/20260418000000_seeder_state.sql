-- seeder cursor persistence for resume capability

CREATE TABLE IF NOT EXISTS seeder_state (
    id         INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    key        TEXT NOT NULL UNIQUE,
    value      TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW()
);

INSERT INTO seeder_state (key, value) VALUES ('cursor', '') ON CONFLICT DO NOTHING;
