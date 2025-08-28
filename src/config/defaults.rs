use super::types::{Config, SegmentsConfig};

pub const DEFAULT_CONFIG: Config = Config {
    theme: String::new(), // Set to "dark" at runtime
    first_run: true,
    jwt_token: None, // JWT token to be set by user
    segments: SegmentsConfig {
        directory: true,
        git: true,
        model: true,
        usage: true,
        quota: true, // Enabled by default
        time: false,
        emoji: false,
        spinner: true,
        network: false, // Network segment disabled by default
        ranking: true, // Ranking segment enabled by default
    },
};

impl Default for Config {
    fn default() -> Self {
        Config {
            theme: "dark".to_string(),
            first_run: true,
            jwt_token: None, // JWT token to be set by user
            segments: SegmentsConfig {
                directory: true,
                git: true,
                model: true,
                usage: true,
                quota: true, // Enabled by default
                time: false,
                emoji: false,
                spinner: true,
                network: false, // Network segment disabled by default
                ranking: true, // Ranking segment enabled by default
            },
        }
    }
}
