-- Initial schema for {{name}}.
-- Prices are integer cents; no floating point money.

CREATE TABLE IF NOT EXISTS products (
    id         TEXT   PRIMARY KEY,
    name       TEXT   NOT NULL,
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0),
    stock      BIGINT NOT NULL CHECK (stock >= 0)
);

CREATE TABLE IF NOT EXISTS orders (
    id          TEXT   PRIMARY KEY,
    product_id  TEXT   NOT NULL REFERENCES products(id),
    quantity    BIGINT NOT NULL CHECK (quantity > 0),
    total_cents BIGINT NOT NULL CHECK (total_cents >= 0),
    status      TEXT   NOT NULL
);
