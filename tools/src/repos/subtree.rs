use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use log::info;
use serde::{Deserialize, Serialize};

use crate::{Result, utils::text_file::TextFile};

#[derive(Debug, Serialize, Deserialize)]
pub struct Subtree {
    pub repo: String,
    pub branch: String,
}

pub struct Subtrees {
    tree: BTreeMap<String, Subtree>,
}

impl Subtrees {
    fn parse(path: &Path) -> Result<Self> {
        let file = TextFile::read(path.into())?;
        let tree = toml::from_str(&file.text)?;
        Ok(Self { tree })
    }

    fn pull_all(&self, repo_root: &PathBuf) -> Result<()> {
        for (name, tree) in &self.tree {
            let prefix = format!("repos/{name}");

            info!(
                "Updating subtree '{prefix}'\nFrom: {} ({})\n",
                tree.repo, tree.branch,
            );

            Command::new("git")
                .current_dir(repo_root)
                .args([
                    "subtree",
                    "pull",
                    "--prefix",
                    &prefix,
                    &tree.repo,
                    &tree.branch,
                    "--squash",
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
        }
        Ok(())
    }
}

pub fn pull_repos(dir: &PathBuf) -> Result<()> {
    Subtrees::parse(&dir.join("./repos/subtree.toml"))?.pull_all(dir)?;
    Ok(())
}
