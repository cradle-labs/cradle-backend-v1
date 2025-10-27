use std::env;

#[derive(Clone)]
pub struct ApiConfig {
    pub secret_key: String,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        let secret_key = env::var("API_SECRET_KEY").unwrap_or_else(|_| {
            tracing::warn!("API_SECRET_KEY not set in environment, using default");
            "default-secret-key".to_string()
        });

        Self { secret_key }
    }
}
