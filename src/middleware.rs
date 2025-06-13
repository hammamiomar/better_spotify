// in src/middleware.rs
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::CookieJar;
use neo4rs::{query, Query, Row};
use std::sync::Arc;

use crate::server::AppState;

// This struct will hold the context of the currently logged-in user.
// We can extract this in any of our server functions to know who is making the request.
#[derive(Clone)]
pub struct UserContext {
    pub spotify_id: String,
}

// This is the core of the middleware. We're telling Axum how to create
// a UserContext from the parts of an incoming HTTP request.
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
        let session_id = if let Some(cookie) = jar.get("sid") {
            cookie.value().to_string()
        } else {
            // If there's no cookie, the user is not logged in. Reject and redirect.
            return Err(Redirect::temporary("/login?reason=no_session"));
        };

        // 3. Query the database to see if this is a valid, unexpired session.
        let mut query = query(
            "MATCH (u:User)-[:HAS_SESSION]->(s:Session {session_id: $sid})
             WHERE s.expires_at > datetime()
             RETURN u.spotify_id AS spotify_id",
        );
        query = query.param("sid", session_id);

        // Run the query and get the first row of results.
        let mut result = state.db.execute(query).await.map_err(|e| {
            tracing::error!("Database error during auth: {}", e);
            Redirect::temporary("/login?error=db_error")
        })?;

        // 4. Check the query result.
        if let Ok(Some(row)) = result.next().await {
            // If we got a row, the session is valid! Extract the spotify_id.
            let spotify_id: String = row.get("spotify_id").unwrap();
            Ok(UserContext { spotify_id })
        } else {
            // If we got no rows, the session is invalid or expired. Reject and redirect.
            Err(Redirect::temporary("/login?reason=invalid_session"))
        }
    }
}