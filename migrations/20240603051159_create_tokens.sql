-- Add migration script here

CREATE TABLE IF NOT EXISTS tokens (
    scope TEXT NOT NULL PRIMARY KEY,

    access_token TEXT NOT NULL,
    api_domain TEXT NOT NULL,
    expires_in BIGINT NOT NULL,
    refresh_token TEXT NOT NULL,
    token_type TEXT NOT NULL,
    time_stamp TIMESTAMPTZ NOT NULL
);
