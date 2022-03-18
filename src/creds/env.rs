use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to get env var {name:?}: {source}")]
pub struct GetEnvError {
    name: String,
    source: std::env::VarError,
}

fn get_env(name: &str) -> Result<String, GetEnvError> {
    std::env::var(name).map_err(|source| GetEnvError {
        name: name.to_owned(),
        source,
    })
}

#[derive(Debug)]
pub struct Env {
    pub app_id: String,
    pub app_key: String,
}

impl Env {
    pub fn read() -> Result<Self, GetEnvError> {
        Ok(Self {
            app_id: get_env("GITHUB_APP_ID")?,
            app_key: get_env("GITHUB_APP_KEY")?,
        })
    }
}
