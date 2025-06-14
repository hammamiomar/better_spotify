pub mod pkce{
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use sha2::{Digest,Sha256};

    pub fn generate_code_verifier() -> String{
        let mut rng = thread_rng();
        let verifier : String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(128)
            .collect();
        verifier
    }

    pub fn generate_code_challenge(verifier: &str) -> String{
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
    }
}

/// Authentication helpers for server functions
#[cfg(feature = "server")]
pub mod helpers {
    use dioxus::prelude::*;
    use neo4rs::query;
    use crate::server::AppState;
    
    /// Simple struct to hold authenticated user info
    pub struct AuthenticatedUser {
        pub spotify_id: String,
        pub access_token: String,
        pub refresh_token: Option<String>,
    }
    
    /// Get the current authenticated user from cookies
    /// Returns None if not authenticated
    pub async fn get_current_user() -> Option<AuthenticatedUser> {
        // Extract cookie from request headers
        let context = dioxus::prelude::server_context();
        let parts = context.request_parts();
        let cookie_header = parts.headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())?;
        
        // Find the session cookie
        let session_id = cookie_header
            .split(';')
            .find(|c| c.trim().starts_with("sid="))
            .map(|c| c.trim().trim_start_matches("sid=").to_string())?;
        
        // Get app state
        let FromContext(app_state) = extract::<FromContext<AppState>, ()>().await.ok()?;
        
        // Query the database for the session
        let mut query = query(
            "MATCH (u:User)-[:HAS_SESSION]->(s:Session {session_id: $sid})
             WHERE s.expires_at > datetime()
             RETURN u.spotify_id AS spotify_id, 
                    u.access_token AS access_token,
                    u.refresh_token AS refresh_token"
        );
        query = query.param("sid", session_id);
        
        let mut result = app_state.db.execute(query).await.ok()?;
        let row = result.next().await.ok()??;
        
        Some(AuthenticatedUser {
            spotify_id: row.get("spotify_id").ok()?,
            access_token: row.get("access_token").ok()?,
            refresh_token: row.get("refresh_token").ok(),
        })
    }
    
    /// Require authentication - returns error if not authenticated
    pub async fn require_auth() -> Result<AuthenticatedUser, ServerFnError> {
        get_current_user().await
            .ok_or_else(|| ServerFnError::ServerError("Authentication required".to_string()))
    }
}