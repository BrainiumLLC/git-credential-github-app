mod auth;
mod env;
mod installation;
mod writer;

use self::{env::*, installation::*, writer::*};
use octocrab::{auth::AppAuth, OctocrabBuilder};
use serde::Serialize;
use std::time::Duration;

static OWNER: &str = "BrainiumLLC";
// 60 minutes is the actual expiration on GitHub, but we need to have a margin
// to ensure the token doesn't expire mid-build.
const MAX_TOKEN_AGE: Duration = Duration::from_secs(45 * 60);

#[derive(Debug, Serialize)]
pub struct UsernamePasswordPair {
    username: String,
    password: String,
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let writer = {
        let dst_dir = std::env::temp_dir().join(concat!("com.brainium.", env!("CARGO_PKG_NAME")));
        GitAskpassWriter::new(&dst_dir)?
    };
    if writer.needs_refresh(MAX_TOKEN_AGE)? {
        log::info!("a new `GIT_ASKPASS` file must be generated");
        let creds = UsernamePasswordPair::from_github_app().await?;
        writer.write(&creds)?;
    } else {
        log::info!("current `GIT_ASKPASS` file is still valid");
    }
    println!("{}", writer.dst().display());
    Ok(())
}
