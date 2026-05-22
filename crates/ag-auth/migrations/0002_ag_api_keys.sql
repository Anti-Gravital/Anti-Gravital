-- API keys almacenadas por su hash BLAKE3.
-- Solo el hash se persiste; la clave en texto plano se entrega una unica vez al usuario.
CREATE TABLE IF NOT EXISTS ag_api_keys (
    id           UUID PRIMARY KEY,
    user_id      UUID        NOT NULL,
    key_hash     TEXT        NOT NULL UNIQUE,
    name         TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,
    revoked      BOOLEAN     NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS ag_api_keys_user_id_idx  ON ag_api_keys (user_id);
CREATE INDEX IF NOT EXISTS ag_api_keys_key_hash_idx ON ag_api_keys (key_hash);
