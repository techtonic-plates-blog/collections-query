use std::sync::Arc;

use crate::auth::{self, Claims};
use axum::http::HeaderMap;
use juniper::Context as JuniperContext;
use sea_orm::DatabaseConnection;

pub type AppState = Arc<AppData>;

pub fn extract_user_from_headers(headers: &HeaderMap) -> Option<Claims> {
    let auth_header = headers.get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    
    if !auth_str.starts_with("Bearer ") {
        return None;
    }
    
    let token = &auth_str[7..];
    auth::verify_token(token.to_string())
}

#[derive(Clone)]
pub struct AppData {
    pub db: DatabaseConnection,
    pub claims: Option<Claims>,
}

impl JuniperContext for AppData {}

impl AppData {
    pub fn new(db: DatabaseConnection, current_user: Option<Claims>) -> Self {
        Self { db, claims: current_user }
    }

    /// Get the current authenticated user or return an error
    pub fn require_auth(&self) -> juniper::FieldResult<&Claims> {
        self.claims
            .as_ref()
            .ok_or_else(|| juniper::FieldError::new(
                "Authentication required",
                juniper::graphql_value!({ "code": "UNAUTHENTICATED" })
            ))
    }

}