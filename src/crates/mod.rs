mod git;
mod local;
mod registry;

use crate::Workspace;
use failure::Error;
use log::info;
use std::path::Path;

pub use registry::AlternativeRegistry;

trait CrateTrait: std::fmt::Display {
    fn fetch(&self, workspace: &Workspace) -> Result<(), Error>;
    fn purge_from_cache(&self, workspace: &Workspace) -> Result<(), Error>;
    fn copy_source_to(&self, workspace: &Workspace, dest: &Path) -> Result<(), Error>;
}

enum CrateType {
    Registry(registry::RegistryCrate),
    Git(git::GitRepo),
    Local(local::Local),
}

/// A Rust crate that can be used with rustwide.
pub struct Crate(CrateType);

impl Crate {
    /// Get the name of the crate
    pub fn name(&self) -> &str {
        match &self.0 {
            CrateType::Registry(krate) => &krate.name,
            CrateType::Git(repo) => &repo.name,
            CrateType::Local(local) => &local.name,
        }
    }

    /// Load a crate from specified registry.
    pub fn registry(registry: AlternativeRegistry, name: &str, version: &str) -> Self {
        Crate(CrateType::Registry(registry::RegistryCrate::new(
            registry::Registry::Alternative(registry),
            name,
            version,
        )))
    }

    /// Load a crate from the [crates.io registry](https://crates.io).
    pub fn crates_io(name: &str, version: &str) -> Self {
        Crate(CrateType::Registry(registry::RegistryCrate::new(
            registry::Registry::CratesIo,
            name,
            version,
        )))
    }

    /// Load a crate from a git repository. The full URL needed to clone the repo has to be
    /// provided.
    pub fn git(url: &str, name: &str) -> Self {
        Crate(CrateType::Git(git::GitRepo::new(url, name)))
    }

    /// Load a crate from a git repository targeting a specific branch. The full URL needed to
    /// clone the repo has to be provided.
    pub fn git_branch(url: &str, name: &str, branch: &str) -> Self {
        Crate(CrateType::Git(
            git::GitRepo::new(url, name).branch(Some(branch)),
        ))
    }

    /// Load a crate from a directory in the local filesystem.
    pub fn local(path: &Path, name: &str) -> Self {
        Crate(CrateType::Local(local::Local::new(path, name)))
    }

    /// Fetch the crate's source code and cache it in the workspace. This method will reach out to
    /// the network for some crate types.
    pub fn fetch(&self, workspace: &Workspace) -> Result<(), Error> {
        self.as_trait().fetch(workspace)
    }

    /// Remove the cached copy of this crate. The method will do nothing if the crate isn't cached.
    pub fn purge_from_cache(&self, workspace: &Workspace) -> Result<(), Error> {
        self.as_trait().purge_from_cache(workspace)
    }

    /// Get this crate's git commit. This method is best-effort, and currently works just for git
    /// crates. If the commit can't be retrieved `None` will be returned.
    pub fn git_commit(&self, workspace: &Workspace) -> Option<String> {
        if let CrateType::Git(repo) = &self.0 {
            repo.git_commit(workspace)
        } else {
            None
        }
    }

    pub(crate) fn copy_source_to(&self, workspace: &Workspace, dest: &Path) -> Result<(), Error> {
        if dest.exists() {
            info!(
                "crate source directory {} already exists, cleaning it up",
                dest.display()
            );
            crate::utils::remove_dir_all(dest)?;
        }
        self.as_trait().copy_source_to(workspace, dest)
    }

    fn as_trait(&self) -> &dyn CrateTrait {
        match &self.0 {
            CrateType::Registry(krate) => krate,
            CrateType::Git(repo) => repo,
            CrateType::Local(local) => local,
        }
    }
}

impl std::fmt::Display for Crate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_trait())
    }
}
