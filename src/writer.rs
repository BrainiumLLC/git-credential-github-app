use crate::UsernamePasswordPair;
use std::{path::Path, time::Duration};

#[derive(Debug)]
pub struct GitAskpassWriter {
    action: bicycle::Action,
}

impl GitAskpassWriter {
    pub fn new(dst_dir: &Path) -> anyhow::Result<Self> {
        static SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/git-askpass.sh.hbs");
        let action = bicycle::Action::detect(
            SRC,
            dst_dir,
            bicycle::no_transform,
            bicycle::DEFAULT_TEMPLATE_EXT,
        )?;
        Ok(Self { action })
    }

    pub fn dst(&self) -> &Path {
        self.action.dst()
    }

    pub fn needs_refresh(&self, max_age: Duration) -> anyhow::Result<bool> {
        if self.dst().is_file() {
            let age = std::fs::metadata(self.dst())?.modified()?.elapsed()?;
            log::info!(
                "current `GIT_ASKPASS` file is {} minutes old",
                age.as_secs() / 60
            );
            Ok(age >= max_age)
        } else {
            log::info!("no `GIT_ASKPASS` file currently exists");
            Ok(true)
        }
    }

    fn chmod(&self) -> bossy::Result<()> {
        bossy::Command::impure_parse("chmod +x")
            .with_arg(self.dst())
            .run_and_wait()?;
        Ok(())
    }

    pub fn write(&self, creds: &UsernamePasswordPair) -> anyhow::Result<()> {
        let bike = bicycle::Bicycle::default();
        bike.process_action(&self.action, |map| {
            map.insert("creds", creds);
        })?;
        self.chmod()?;
        Ok(())
    }
}
