-- Sesiones con refresh tokens y rotacion.
-- JTI identifica de forma unica cada token de refresco.
-- Al rotar, el JTI anterior queda revocado (revoked = true).
CREATE TABLE IF NOT EXISTS ag_sessions (
    jti        UUID PRIMARY KEY,
    user_id    UUID        NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked    BOOLEAN     NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ag_sessions_user_id_idx ON ag_sessions (user_id);
CREATE INDEX IF NOT EXISTS ag_sessions_expires_at_idx ON ag_sessions (expires_at);
