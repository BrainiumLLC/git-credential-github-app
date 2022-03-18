use super::env::Env;
use jsonwebtoken::EncodingKey;
use octocrab::{auth::AppAuth, models::AppId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppAuthFromEnvError {
    #[error("App ID {app_id:?} wasn't a lovely positive integer: {source}")]
    AppIdInvalid {
        app_id: String,
        source: std::num::ParseIntError,
    },
    #[error("App key wasn't a valid RSA key: {0}")]
    AppKeyInvalid(#[from] jsonwebtoken::errors::Error),
}

impl TryFrom<Env> for AppAuth {
    type Error = AppAuthFromEnvError;

    fn try_from(Env { app_id, app_key }: Env) -> Result<Self, Self::Error> {
        Ok(Self {
            app_id: app_id
                .parse()
                .map(AppId)
                .map_err(|source| AppAuthFromEnvError::AppIdInvalid { app_id, source })?,
            key: EncodingKey::from_rsa_pem(app_key.as_bytes())?,
        })
    }
}
