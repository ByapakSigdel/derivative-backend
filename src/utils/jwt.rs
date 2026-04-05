//! JWT token generation and validation utilities.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::CONFIG;
use crate::errors::{AppError, AppResult};
use crate::models::UserType;

/// JWT claims for access tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User email
    pub email: String,
    /// User type (admin/user)
    pub user_type: UserType,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// Token type identifier
    pub token_type: String,
}

/// JWT claims for refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// Token type identifier
    pub token_type: String,
    /// Unique token ID for revocation
    pub jti: Uuid,
}

/// Token pair returned after authentication
#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
}

/// Generate an access token for a user
pub fn generate_access_token(user_id: Uuid, email: &str, user_type: UserType) -> AppResult<String> {
    let now = Utc::now();
    let exp = now + Duration::seconds(CONFIG.jwt_access_expiry);
    
    let claims = AccessTokenClaims {
        sub: user_id,
        email: email.to_string(),
        user_type,
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "access".to_string(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to generate access token: {}", e)))
}

/// Generate a refresh token for a user
pub fn generate_refresh_token(user_id: Uuid) -> AppResult<(String, Uuid)> {
    let now = Utc::now();
    let exp = now + Duration::seconds(CONFIG.jwt_refresh_expiry);
    let jti = Uuid::new_v4();
    
    let claims = RefreshTokenClaims {
        sub: user_id,
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type: "refresh".to_string(),
        jti,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to generate refresh token: {}", e)))?;
    
    Ok((token, jti))
}

/// Generate both access and refresh tokens
pub fn generate_token_pair(user_id: Uuid, email: &str, user_type: UserType) -> AppResult<TokenPair> {
    let access_token = generate_access_token(user_id, email, user_type)?;
    let (refresh_token, _jti) = generate_refresh_token(user_id)?;
    
    Ok(TokenPair {
        access_token,
        refresh_token,
        access_token_expires_in: CONFIG.jwt_access_expiry,
        refresh_token_expires_in: CONFIG.jwt_refresh_expiry,
    })
}

/// Verify and decode an access token
pub fn verify_access_token(token: &str) -> AppResult<AccessTokenClaims> {
    let token_data: TokenData<AccessTokenClaims> = decode(
        token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        }
    })?;
    
    if token_data.claims.token_type != "access" {
        return Err(AppError::InvalidToken);
    }
    
    Ok(token_data.claims)
}

/// Verify and decode a refresh token
pub fn verify_refresh_token(token: &str) -> AppResult<RefreshTokenClaims> {
    let token_data: TokenData<RefreshTokenClaims> = decode(
        token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        }
    })?;
    
    if token_data.claims.token_type != "refresh" {
        return Err(AppError::InvalidToken);
    }
    
    Ok(token_data.claims)
}

/// Extract token from Authorization header
pub fn extract_bearer_token(header_value: &str) -> Option<&str> {
    header_value.strip_prefix("Bearer ").or_else(|| header_value.strip_prefix("bearer "))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_access_token_roundtrip() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let user_type = UserType::User;
        
        let token = generate_access_token(user_id, email, user_type).unwrap();
        let claims = verify_access_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.user_type, user_type);
    }
    
    #[test]
    fn test_refresh_token_roundtrip() {
        let user_id = Uuid::new_v4();
        
        let (token, jti) = generate_refresh_token(user_id).unwrap();
        let claims = verify_refresh_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.jti, jti);
    }
    
    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(extract_bearer_token("Bearer token123"), Some("token123"));
        assert_eq!(extract_bearer_token("bearer token123"), Some("token123"));
        assert_eq!(extract_bearer_token("Basic token123"), None);
    }
}
