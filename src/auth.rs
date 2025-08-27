
use jsonwebtoken::{decode, Algorithm, Validation};
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;


/// Structured permission with action, resource, and scope
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Permission {
    pub action: String,
    pub resource: String,
    pub scope: String,
}

impl Permission {
    pub fn new(action: &str, resource: &str, scope: &str) -> Self {
        Self {
            action: action.to_string(),
            resource: resource.to_string(),
            scope: scope.to_string(),
        }
    }

    /// Parse from "action:resource:scope" format
    pub fn from_string(permission_str: &str) -> Option<Self> {
        let parts: Vec<&str> = permission_str.split(':').collect();
        if parts.len() == 3 {
            Some(Self::new(parts[0], parts[1], parts[2]))
        } else {
            None
        }
    }

    /// Convert to "action:resource:scope" format
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.action, self.resource, self.scope)
    }
}

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
    pub permissions: Vec<Permission>,
}

impl Claims {
    /// Check if the user has a specific permission
    pub fn has_permission(&self, action: &str, resource: &str) -> bool {
        // Check for both "any" scope and "owned" scope permissions
        let any_permission = Permission::new(action, resource, "any");
        let owned_permission = Permission::new(action, resource, "owned");
        self.permissions.contains(&any_permission) || self.permissions.contains(&owned_permission)
    }

    /// Check if the user has a specific permission with a specific scope
    pub fn has_permission_with_scope(&self, action: &str, resource: &str, scope: &str) -> bool {
        let required_permission = Permission::new(action, resource, scope);
        self.permissions.contains(&required_permission)
    }

    /// Check if the user has any of the specified permissions
    pub fn has_any_permission(&self, permissions: &[(String, String)]) -> bool {
        permissions.iter().any(|(action, resource)| {
            self.has_permission(action, resource)
        })
    }

    /// Check if the user has any of the specified permissions with specific scopes
    pub fn has_any_permission_with_scope(&self, permissions: &[(String, String, String)]) -> bool {
        permissions.iter().any(|(action, resource, scope)| {
            self.has_permission_with_scope(action, resource, scope)
        })
    }
}


pub fn verify_token(token: String) -> Option<Claims> {
    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(CONFIG.jwt_public_key.as_bytes()).ok()?;
    let Ok(token) = decode(
        &token,
        &decoding_key,
        &Validation::new(Algorithm::RS256),
    ) else {
        return None;
    };
    Some(token.claims)
}

pub fn assert_logged_in(user: &Option<Claims>) -> juniper::FieldResult<&Claims> {
    user.as_ref().ok_or_else(|| juniper::FieldError::new(
        "Authentication required",
        juniper::graphql_value!({ "code": "UNAUTHENTICATED" })
    ))
}