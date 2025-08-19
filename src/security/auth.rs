use anyhow::{Result, anyhow};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    sub: String,  // Subject (user ID)
    exp: usize,   // Expiration time
    iat: usize,   // Issued at
    jti: String,  // JWT ID
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub expires_at: SystemTime,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
struct User {
    username: String,
    password_hash: String,
    permissions: Vec<String>,
    created_at: SystemTime,
}

pub struct AuthManager {
    users: RwLock<HashMap<String, User>>,
    revoked_tokens: RwLock<HashMap<String, SystemTime>>,
    jwt_secret: Vec<u8>,
}

impl AuthManager {
    pub fn new() -> Self {
        // Generate a random JWT secret (in production, this should be loaded from config)
        let jwt_secret = (0..32).map(|_| rand::random::<u8>()).collect();
        
        Self {
            users: RwLock::new(HashMap::new()),
            revoked_tokens: RwLock::new(HashMap::new()),
            jwt_secret,
        }
    }

    pub async fn generate_token(&self, username: &str, duration: Duration) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;
        
        let exp = now + duration.as_millis() as usize;
        let jti = Uuid::new_v4().to_string();
        
        let claims = Claims {
            sub: username.to_string(),
            exp,
            iat: now,
            jti,
        };
        
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(&self.jwt_secret);
        
        encode(&header, &claims, &encoding_key)
            .map_err(|e| anyhow!("Failed to generate token: {}", e))
    }

    pub async fn validate_token(&self, token: &str) -> Result<UserInfo> {
        let decoding_key = DecodingKey::from_secret(&self.jwt_secret);
        let validation = Validation::default();
        
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow!("invalid token: {}", e))?;
        
        let claims = token_data.claims;
        
        // Check if token is revoked
        let revoked_tokens = self.revoked_tokens.read().await;
        if revoked_tokens.contains_key(&claims.jti) {
            return Err(anyhow!("token revoked"));
        }
        
        // Check expiration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;
        
        if claims.exp < now {
            return Err(anyhow!("token expired"));
        }
        
        // Get user permissions
        let users = self.users.read().await;
        let permissions = users.get(&claims.sub)
            .map(|user| user.permissions.clone())
            .unwrap_or_default();
        
        Ok(UserInfo {
            username: claims.sub,
            expires_at: UNIX_EPOCH + Duration::from_millis(claims.exp as u64),
            permissions,
        })
    }

    pub async fn add_user(&mut self, username: &str, password: &str, permissions: Vec<String>) -> Result<()> {
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
        
        let user = User {
            username: username.to_string(),
            password_hash,
            permissions,
            created_at: SystemTime::now(),
        };
        
        let mut users = self.users.write().await;
        users.insert(username.to_string(), user);
        
        Ok(())
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<String> {
        let users = self.users.read().await;
        let user = users.get(username)
            .ok_or_else(|| anyhow!("user not found"))?;
        
        let valid = verify(password, &user.password_hash)
            .map_err(|e| anyhow!("Password verification failed: {}", e))?;
        
        if !valid {
            return Err(anyhow!("authentication failed"));
        }
        
        // Generate token for authenticated user
        self.generate_token(username, Duration::from_secs(3600)).await
    }

    pub async fn refresh_token(&self, old_token: &str, new_duration: Duration) -> Result<String> {
        let user_info = self.validate_token(old_token).await?;
        self.generate_token(&user_info.username, new_duration).await
    }

    pub async fn revoke_token(&mut self, token: &str) -> Result<()> {
        let decoding_key = DecodingKey::from_secret(&self.jwt_secret);
        let validation = Validation::default();
        
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow!("invalid token: {}", e))?;
        
        let mut revoked_tokens = self.revoked_tokens.write().await;
        revoked_tokens.insert(token_data.claims.jti, SystemTime::now());
        
        Ok(())
    }

    pub async fn get_user_password_hash(&self, username: &str) -> Result<String> {
        let users = self.users.read().await;
        let user = users.get(username)
            .ok_or_else(|| anyhow!("user not found"))?;
        
        Ok(user.password_hash.clone())
    }
}