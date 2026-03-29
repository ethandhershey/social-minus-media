CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "vector";
CREATE EXTENSION IF NOT EXISTS "cube";
CREATE EXTENSION IF NOT EXISTS "earthdistance";

CREATE TABLE IF NOT EXISTS users (
    id                      UUID PRIMARY KEY NOT NULL,
    sub                     TEXT UNIQUE NOT NULL,
    email                   TEXT NOT NULL,
    display_name            TEXT NOT NULL,
    avatar_url              TEXT,
    bio                     TEXT,
    city                    TEXT,
    latitude                DOUBLE PRECISION,
    longitude               DOUBLE PRECISION,
    tier                    TEXT NOT NULL DEFAULT 'Free',
    api_usage               BIGINT NOT NULL DEFAULT 0,
    storage_usage           BIGINT NOT NULL DEFAULT 0,
    billing_customer_id     TEXT,
    billing_period_start    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at             TIMESTAMPTZ
);

CREATE UNIQUE INDEX users_sub_idx ON users (sub);
CREATE INDEX idx_users_location ON users USING GIST (ll_to_earth(latitude, longitude)) WHERE latitude IS NOT NULL AND longitude IS NOT NULL;

CREATE TABLE IF NOT EXISTS products (
    id          UUID PRIMARY KEY NOT NULL,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    brand       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ
);

CREATE INDEX idx_products_user_id ON products(user_id);

CREATE TABLE IF NOT EXISTS product_insights (
    id              UUID PRIMARY KEY NOT NULL,
    product_id      UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    content         TEXT NOT NULL,
    generated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_insights_product_id ON product_insights(product_id);
CREATE INDEX idx_insights_product_id_generated_at ON product_insights(product_id, generated_at);

CREATE TABLE IF NOT EXISTS user_interests (
    id          UUID PRIMARY KEY NOT NULL,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    messages    JSONB NOT NULL DEFAULT '[]',
    summary     TEXT,
    embedding   VECTOR(1024),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_interests_user_id ON user_interests(user_id);

CREATE TABLE IF NOT EXISTS events (
    id              UUID PRIMARY KEY NOT NULL,
    host_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT,
    address         TEXT,
    latitude        DOUBLE PRECISION,
    longitude       DOUBLE PRECISION,
    start_time      TIMESTAMPTZ NOT NULL,
    max_capacity    INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_events_host_id ON events(host_id);

CREATE TABLE IF NOT EXISTS rsvps (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_id    UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    status      TEXT NOT NULL CHECK (status IN ('going', 'maybe', 'declined')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, event_id)
);

CREATE INDEX idx_rsvps_event_id ON rsvps(event_id);
CREATE INDEX idx_rsvps_user_id ON rsvps(user_id);
