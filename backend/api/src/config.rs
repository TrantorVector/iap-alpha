use std::env;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Development => write!(f, "Development"),
            Self::Staging => write!(f, "Staging"),
            Self::Production => write!(f, "Production"),
        }
    }
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            _ => Ok(Self::Development),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    MissingEnv(String),
    InvalidFormat(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv(var) => write!(f, "Missing environment variable: {}", var),
            Self::InvalidFormat(msg) => write!(f, "Invalid configuration format: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_private_key_file: Option<String>,
    pub jwt_public_key_file: Option<String>,
    pub jwt_private_key: Option<String>,
    pub jwt_public_key: Option<String>,
    pub server_host: String,
    pub server_port: u16,
    pub cors_origins: Vec<String>,
    pub alpha_vantage_api_key: Option<String>,
    pub s3_endpoint: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub environment: Environment,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"[REDACTED]")
            .field("jwt_private_key_file", &self.jwt_private_key_file)
            .field("jwt_public_key_file", &self.jwt_public_key_file)
            .field(
                "jwt_private_key",
                &self.jwt_private_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "jwt_public_key",
                &self.jwt_public_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("server_host", &self.server_host)
            .field("server_port", &self.server_port)
            .field("cors_origins", &self.cors_origins)
            .field(
                "alpha_vantage_api_key",
                &self.alpha_vantage_api_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("s3_endpoint", &self.s3_endpoint)
            .field("s3_access_key", &"[REDACTED]")
            .field("s3_secret_key", &"[REDACTED]")
            .field("environment", &self.environment)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = match env::var("ENVIRONMENT").or_else(|_| env::var("RUST_ENV")) {
            Ok(val) => val.parse().unwrap_or(Environment::Development),
            Err(_) => Environment::Development,
        };

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnv("DATABASE_URL".to_string()))?;

        let jwt_private_key_file = env::var("JWT_PRIVATE_KEY_FILE")
            .or_else(|_| env::var("JWT_PRIVATE_KEY_PATH"))
            .ok();
        let jwt_public_key_file = env::var("JWT_PUBLIC_KEY_FILE")
            .or_else(|_| env::var("JWT_PUBLIC_KEY_PATH"))
            .ok();
        let jwt_private_key = env::var("JWT_PRIVATE_KEY").ok();
        let jwt_public_key = env::var("JWT_PUBLIC_KEY").ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidFormat("SERVER_PORT must be a number".to_string()))?;

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let alpha_vantage_api_key = env::var("ALPHA_VANTAGE_API_KEY").ok();

        // Defaults for development
        let s3_endpoint = if let Ok(v) = env::var("S3_ENDPOINT") {
            v
        } else if environment == Environment::Development {
            "http://localhost:9000".to_string()
        } else {
            return Err(ConfigError::MissingEnv("S3_ENDPOINT".to_string()));
        };

        let s3_access_key = if let Ok(v) = env::var("S3_ACCESS_KEY") {
            v
        } else if environment == Environment::Development {
            "minioadmin".to_string()
        } else {
            return Err(ConfigError::MissingEnv("S3_ACCESS_KEY".to_string()));
        };

        let s3_secret_key = if let Ok(v) = env::var("S3_SECRET_KEY") {
            v
        } else if environment == Environment::Development {
            "minioadmin".to_string()
        } else {
            return Err(ConfigError::MissingEnv("S3_SECRET_KEY".to_string()));
        };

        Ok(Config {
            database_url,
            jwt_private_key_file,
            jwt_public_key_file,
            jwt_private_key,
            jwt_public_key,
            server_host,
            server_port,
            cors_origins,
            alpha_vantage_api_key,
            s3_endpoint,
            s3_access_key,
            s3_secret_key,
            environment,
        })
    }
}
