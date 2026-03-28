CREATE TABLE IF NOT EXISTS users (
    id                      TEXT PRIMARY KEY NOT NULL,
    sub                     TEXT UNIQUE NOT NULL,
    email                   TEXT NOT NULL,
    display_name            TEXT NOT NULL,
    tier                    TEXT NOT NULL DEFAULT 'Free',
    api_usage               INTEGER NOT NULL DEFAULT 0,
    storage_usage           INTEGER NOT NULL DEFAULT 0,
    billing_customer_id     TEXT,
    billing_period_start    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    archived_at             TEXT
);

CREATE UNIQUE INDEX users_sub_idx ON users (sub);

CREATE TABLE IF NOT EXISTS products (
    id          TEXT PRIMARY KEY NOT NULL,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    brand       TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    archived_at TEXT
);

CREATE INDEX idx_products_user_id ON products(user_id);

CREATE TABLE IF NOT EXISTS product_insights (
    id              TEXT PRIMARY KEY NOT NULL,
    product_id      TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    content         TEXT NOT NULL, -- JSON
    generated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_insights_product_id ON product_insights(product_id);
CREATE INDEX idx_insights_product_id_generated_at ON product_insights(product_id, generated_at);
