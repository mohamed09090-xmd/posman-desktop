use std::{fs, path::PathBuf};

use crate::error::RuntimeError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimePaths {
    pub root: PathBuf,
    pub data: PathBuf,
    pub backups: PathBuf,
    pub documents: PathBuf,
    pub templates: PathBuf,
    pub logs: PathBuf,
    pub database: PathBuf,
}

impl RuntimePaths {
    pub fn from_root(root: PathBuf) -> Self {
        let data = root.join("data");
        Self {
            backups: root.join("backups"),
            documents: root.join("documents"),
            templates: root.join("templates"),
            logs: root.join("logs"),
            database: data.join("posman.sqlite3"),
            root,
            data,
        }
    }

    pub fn create_all(root: PathBuf) -> Result<Self, RuntimeError> {
        let paths = Self::from_root(root);
        for directory in paths.directories() {
            fs::create_dir_all(directory).map_err(|source| RuntimeError::PathCreation {
                path: directory.clone(),
                source,
            })?;
        }
        Ok(paths)
    }

    fn directories(&self) -> [&PathBuf; 6] {
        [
            &self.root,
            &self.data,
            &self.backups,
            &self.documents,
            &self.templates,
            &self.logs,
        ]
    }
}
