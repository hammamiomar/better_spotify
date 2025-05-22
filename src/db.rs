use sqlx::{FromRow, SqlitePool};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub spotify_user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserToken {
    pub id: String,
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PkceVerifier {
    pub id: String,
    pub state: String,
    pub code_verifier: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create database file if it doesn't exist for SQLite
        if database_url.starts_with("sqlite:") {
            let db_path = database_url.strip_prefix("sqlite:").unwrap_or("data.db");
            if let Some(parent) = std::path::Path::new(db_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        let pool = SqlitePool::connect(database_url).await?;
        
        // Run migrations - use relative path from project root
        if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
            tracing::warn!("Migration failed, creating tables manually: {}", e);
            // Fallback: create tables manually
            Self::create_tables_manually(&pool).await?;
        }
        
        Ok(Self { pool })
    }
    
    async fn create_tables_manually(pool: &SqlitePool) -> Result<()> {
        // Create tables manually if migrations fail
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                spotify_user_id TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_users_spotify_id ON users(spotify_user_id);
            
            CREATE TABLE IF NOT EXISTS user_tokens (
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
            
            CREATE INDEX IF NOT EXISTS idx_user_tokens_user_id ON user_tokens(user_id);
            CREATE INDEX IF NOT EXISTS idx_user_tokens_expires ON user_tokens(expires_at);
            
            CREATE TABLE IF NOT EXISTS pkce_verifiers (
                id TEXT PRIMARY KEY,
                state TEXT NOT NULL UNIQUE,
                code_verifier TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_pkce_state ON pkce_verifiers(state);
            CREATE INDEX IF NOT EXISTS idx_pkce_expires ON pkce_verifiers(expires_at);
            "#
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // User operations
    pub async fn create_user(&self, spotify_user_id: &str) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, spotify_user_id, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(spotify_user_id)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn get_user_by_spotify_id(&self, spotify_user_id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE spotify_user_id = ?"
        )
        .bind(spotify_user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    // Token operations
    pub async fn store_user_token(&self, user_id: &str, token_data: &crate::api_models::SpotifyTokenResponse) -> Result<UserToken> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(token_data.expires_in as i64);
        
        // Delete existing tokens for this user
        sqlx::query("DELETE FROM user_tokens WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        
        let token = sqlx::query_as::<_, UserToken>(
            r#"
            INSERT INTO user_tokens (id, user_id, access_token, refresh_token, token_type, expires_at, scope, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(&token_data.access_token)
        .bind(&token_data.refresh_token)
        .bind(&token_data.token_type)
        .bind(expires_at)
        .bind(&token_data.scope.as_ref().unwrap_or(&String::new()))
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(token)
    }

    pub async fn get_user_token(&self, user_id: &str) -> Result<Option<UserToken>> {
        let token = sqlx::query_as::<_, UserToken>(
            "SELECT * FROM user_tokens WHERE user_id = ? AND expires_at > ? ORDER BY created_at DESC LIMIT 1"
        )
        .bind(user_id)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(token)
    }

    // PKCE operations
    pub async fn store_pkce_verifier(&self, state: &str, code_verifier: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::minutes(10); // PKCE verifiers expire in 10 minutes
        
        sqlx::query(
            r#"
            INSERT INTO pkce_verifiers (id, state, code_verifier, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(state)
        .bind(code_verifier)
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_and_remove_pkce_verifier(&self, state: &str) -> Result<Option<String>> {
        let now = Utc::now();
        
        // Get the verifier if it exists and hasn't expired
        let verifier = sqlx::query_as::<_, PkceVerifier>(
            "SELECT * FROM pkce_verifiers WHERE state = ? AND expires_at > ?"
        )
        .bind(state)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(v) = verifier {
            // Remove it from the database
            sqlx::query("DELETE FROM pkce_verifiers WHERE id = ?")
                .bind(&v.id)
                .execute(&self.pool)
                .await?;
            
            Ok(Some(v.code_verifier))
        } else {
            Ok(None)
        }
    }

    // Cleanup expired entries
    pub async fn cleanup_expired(&self) -> Result<()> {
        let now = Utc::now();
        
        // Clean up expired PKCE verifiers
        sqlx::query("DELETE FROM pkce_verifiers WHERE expires_at <= ?")
            .bind(now)
            .execute(&self.pool)
            .await?;
        
        // Clean up expired tokens
        sqlx::query("DELETE FROM user_tokens WHERE expires_at <= ?")
            .bind(now)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
}