-- Credenciales WebAuthn (Passkeys) registradas por usuario.
-- credential_id es el identificador opaco devuelto por el autenticador.
-- counter previene ataques de replay (autenticador lo incrementa en cada uso).
CREATE TABLE IF NOT EXISTS ag_webauthn_credentials (
    id            UUID PRIMARY KEY,
    user_id       UUID   NOT NULL,
    credential_id TEXT   NOT NULL UNIQUE,
    public_key    BYTEA  NOT NULL,
    counter       BIGINT NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ag_webauthn_credentials_user_id_idx ON ag_webauthn_credentials (user_id);
