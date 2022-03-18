mod creds;
mod doc;

use self::{creds::*, doc::*};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Exciting GitHub credential helper")]
enum Operation {
    Get,
    Store,
    Erase,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let operation = Operation::from_args();
    log::info!("received operation `{:?}`", operation);
    let doc = Doc::read()?;
    if doc.matches_url("https", "github.com") {
        let cache = Cache::new();
        match operation {
            // git wants fresh-baked credentials! 🍪
            Operation::Get => {
                let creds = UsernamePasswordPair::generate(&cache).await?;
                doc.with_creds(creds).with_quit(true).write()?;
            }
            // git succeeded using the credentials from `get`
            Operation::Store => {
                log::info!("`Store` is unsupported; doing nothing");
            }
            // git failed using the credentials from `get`
            Operation::Erase => {
                if let Some(creds) = UsernamePasswordPair::from_cache(&cache)? {
                    if doc.matches_creds(&creds) {
                        cache.delete()?;
                    } else {
                        log::info!("input creds don't match what's stored; doing nothing");
                    }
                } else {
                    log::info!("no creds are currently stored; doing nothing");
                }
            }
        }
    } else {
        log::info!("request isn't for GitHub; doing nothing");
    }
    Ok(())
}
