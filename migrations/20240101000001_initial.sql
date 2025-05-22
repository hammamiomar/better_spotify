-- Initial migration for user management and tokens

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    spotify_user_id TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_users_spotify_id ON users(spotify_user_id);

CREATE TABLE user_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_type TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_tokens_user_id ON user_tokens(user_id);
CREATE INDEX idx_user_tokens_expires ON user_tokens(expires_at);

CREATE TABLE pkce_verifiers (
    id TEXT PRIMARY KEY,
    state TEXT NOT NULL UNIQUE,
    code_verifier TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE INDEX idx_pkce_state ON pkce_verifiers(state);
CREATE INDEX idx_pkce_expires ON pkce_verifiers(expires_at);