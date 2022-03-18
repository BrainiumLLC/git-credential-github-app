use octocrab::{models::Installation, Octocrab};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FindInstallationError {
    #[error("Failed to get list of installations: {0}")]
    RequestFailed(#[from] octocrab::Error),
    #[error("Failed to find installation for owner {0:?}")]
    NotFound(String),
}

pub async fn find_installation(
    gh: &Octocrab,
    owner: &str,
) -> Result<Installation, FindInstallationError> {
    let mut page_i = 0u32;
    loop {
        let mut page = gh.apps().installations().page(page_i).send().await?;
        log::info!("current page of installations: {}", page_i);
        if let Some(installation) = page
            .take_items()
            .into_iter()
            .find(|installation| installation.account.login == owner)
        {
            break Ok(installation);
        }
        page_i += 1;
        let page_n = page.number_of_pages().unwrap_or(page_i);
        log::info!("total pages of installations: {}", page_n);
        if page_n <= page_i {
            break Err(FindInstallationError::NotFound(owner.to_owned()));
        }
    }
}
