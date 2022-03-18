mod auth;
mod cache;
mod env;
mod installation;

pub use self::cache::*;
use self::{env::*, installation::*};
use octocrab::{auth::AppAuth, OctocrabBuilder};
use serde::{Deserialize, Serialize};

static OWNER: &str = "BrainiumLLC";

#[derive(Debug, Deserialize, Serialize)]
pub struct UsernamePasswordPair {
    pub username: String,
    pub password: String,
}

impl UsernamePasswordPair {
    async fn from_github_app() -> anyhow::Result<Self> {
        let AppAuth { app_id, key } = Env::read()?.try_into()?;
        Ok(Self {
            username: app_id.to_string(),
            password: {
                let gh = OctocrabBuilder::new().app(app_id, key).build()?;
                let installation = find_installation(&gh, OWNER).await?;
                let gh = gh.installation(installation.id);
                gh.request_installation_auth_token().await?
            },
        })
    }

    pub fn from_cache(cache: &Cache) -> Result<Option<Self>, ReadError> {
        cache.read()
    }

    pub async fn generate(cache: &Cache) -> anyhow::Result<Self> {
        if let Some(creds) = Self::from_cache(&cache)? {
            log::info!("current creds are still valid");
            Ok(creds)
        } else {
            log::info!("new creds must be generated");
            let creds = Self::from_github_app().await?;
            cache.write(&creds)?;
            Ok(creds)
        }
    }
}
