use dioxus::prelude::server_fn::error::NoCustomError;
use dioxus::prelude::*;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse,Redirect},
};
use axum_extra::extract::cookie::CookieJar;
use neo4rs::query;

use crate::server::AppState;

// This struct will hold the context of the currently logged-in user.
// We can extract this in any of our server functions to know who is making the request.
#[derive(Clone)]
pub struct UserContext {
    pub spotify_id: String,
}
#[cfg(feature = "server")]
impl UserContext {
    /// Get the user's current access token from the database
    pub async fn get_access_token(&self, db: &neo4rs::Graph) -> Result<String, ServerFnError<NoCustomError>> {
        let query = neo4rs::query(
            "MATCH (u:User {spotify_id: $id})
             RETURN u.access_token as token"
        ).param("id", self.spotify_id.clone());
        
        let mut result = db.execute(query).await
            .map_err(|e| ServerFnError::ServerError::<NoCustomError>(format!("Database error: {}", e)))?;
        
        match result.next().await {
            Ok(Some(row)) => row.get::<String>("token")
                .map_err(|e| ServerFnError::ServerError(format!("Failed to get token: {}", e))),
            Ok(None) => Err(ServerFnError::ServerError("User not found or no token".to_string())),
            Err(e) => Err(ServerFnError::ServerError(format!("Database error: {}", e)))
        }
    }
    
    /// Get the user's refresh token from the database
    pub async fn get_refresh_token(&self, db: &neo4rs::Graph) -> Result<String, ServerFnError<NoCustomError>> {
        let query = neo4rs::query(
            "MATCH (u:User {spotify_id: $id})
             RETURN u.refresh_token as token"
        ).param("id", self.spotify_id.clone());
        
        let mut result = db.execute(query).await
            .map_err(|e| ServerFnError::ServerError::<NoCustomError>(format!("Database error: {}", e)))?;
        
        match result.next().await {
            Ok(Some(row)) => row.get::<String>("token")
                .map_err(|e| ServerFnError::ServerError(format!("Failed to get refresh token: {}", e))),
            Ok(None) => Err(ServerFnError::ServerError("User not found or no refresh token".to_string())),
            Err(e) => Err(ServerFnError::ServerError(format!("Database error: {}", e)))
        }
    }
    
    /// Update the user's access token in the database
    pub async fn update_access_token(&self, db: &neo4rs::Graph, new_token: &str) -> Result<(), ServerFnError<NoCustomError>> {
        let query = neo4rs::query(
            "MATCH (u:User {spotify_id: $id})
             SET u.access_token = $token"
        )
        .param("id", self.spotify_id.clone())
        .param("token", new_token);
        
        db.run(query).await
            .map_err(|e| ServerFnError::ServerError::<NoCustomError>(format!("Failed to update token: {}", e)))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for UserContext {
    // Define the type of error that can occur if authentication fails.
    // In this case, it will be a redirect to the login page.
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Extract the cookie jar from the request. This contains all cookies.
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::temporary("/login?error=cookie_error"))?;

        // 2. Try to get our specific session ID cookie ("sid").
        let session_id = jar
            .get("sid")
            .ok_or_else(|| Redirect::temporary("/login?reason=no_session"))?
            .value()
            .to_string();

        // 3. Query the database to see if this is a valid, unexpired session.
        let mut query = query(
            "MATCH (u:User)-[:HAS_SESSION]->(s:Session {session_id: $sid})
             WHERE s.expires_at > datetime()
             RETURN u.spotify_id AS spotify_id",
        );
        query = query.param("sid", session_id.clone());

        // Run the query and get the first row of results.
        let mut result = state.db.execute(query).await.map_err(|e| {
            tracing::error!("Database error during auth: {}", e);
            Redirect::temporary("/login?error=db_error")
        })?;

        // 4. Check the query result.
        if let Ok(Some(row)) = result.next().await {
            // If we got a row, the session is valid! Extract the spotify_id.
            let spotify_id: String = row.get("spotify_id").map_err(|e| {
                tracing::error!("Failed to get spotify_id from row: {}", e);
                Redirect::temporary("/login?error=data_error")
            })?;
            Ok(UserContext { spotify_id })
        } else {
            // If we got no rows, the session is invalid or expired. Reject and redirect.
            Err(Redirect::temporary("/login?reason=invalid_session"))
        }
    }
}

// Optional user context that doesn't redirect
#[derive(Clone)]
pub struct OptionalUser(pub Option<String>); // Just the spotify_id if authenticated

#[async_trait]
impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookies - if this fails, return None
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(_) => return Ok(OptionalUser(None)),
        };
        
        // Check for session cookie
        let session_id = match jar.get("sid") {
            Some(cookie) => cookie.value().to_string(),
            None => return Ok(OptionalUser(None)),
        };
        
        // Quick database check
        let mut query = query(
            "MATCH (u:User)-[:HAS_SESSION]->(s:Session {session_id: $sid})
             WHERE s.expires_at > datetime()
             RETURN u.spotify_id AS spotify_id",
        );
        query = query.param("sid", session_id);
        
        match state.db.execute(query).await {
            Ok(mut result) => match result.next().await {
                Ok(Some(row)) => match row.get::<String>("spotify_id") {
                    Ok(id) => Ok(OptionalUser(Some(id))),
                    Err(_) => Ok(OptionalUser(None)),
                },
                _ => Ok(OptionalUser(None)),
            },
            Err(_) => Ok(OptionalUser(None)),
        }
    }
}

