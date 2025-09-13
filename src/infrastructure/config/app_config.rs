use std::env;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub base_url: String,
    pub port: u16,
    pub host: String,
    pub environment: Environment,
}

/// Application environment
#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            environment: Environment::Development,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        let environment = match env::var("ENVIRONMENT")?.as_str() {
            "production" => Environment::Production,
            "test" => Environment::Test,
            _ => Environment::Development,
        };

        Ok(AppConfig {
            base_url: env::var("BASE_URL")?,
            port: env::var("PORT")?
                .parse()
                .map_err(|_| env::VarError::NotPresent)?,
            host: env::var("HOST")?,
            environment,
        })
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }
}
