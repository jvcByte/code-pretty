use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub max_file_size: usize,
    pub temp_dir: String,
    pub cors_origins: Vec<String>,
    pub request_timeout_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            max_file_size: 10 * 1024 * 1024, // 10MB
            temp_dir: "/tmp/code-snippet-designer".to_string(),
            cors_origins: vec!["*".to_string()],
            request_timeout_seconds: 30,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(host) = env::var("HOST") {
            config.host = host;
        }

        if let Ok(port) = env::var("PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.port = port_num;
            }
        }

        if let Ok(max_size) = env::var("MAX_FILE_SIZE") {
            if let Ok(size) = max_size.parse::<usize>() {
                config.max_file_size = size;
            }
        }

        if let Ok(temp_dir) = env::var("TEMP_DIR") {
            config.temp_dir = temp_dir;
        }

        if let Ok(origins) = env::var("CORS_ORIGINS") {
            config.cors_origins = origins.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(timeout) = env::var("REQUEST_TIMEOUT_SECONDS") {
            if let Ok(timeout_num) = timeout.parse::<u64>() {
                config.request_timeout_seconds = timeout_num;
            }
        }

        config
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}