use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(crate) const MAX_RECENT_PROJECTS: usize = 5;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct RecentProject {
    pub path: PathBuf,
    pub last_opened: String,
}

/// Pure in-memory list of recent projects with NO filesystem access and NO clock dependency.
pub(crate) struct RecentProjectsList {
    pub projects: Vec<RecentProject>,
}

impl Default for RecentProjectsList {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
        }
    }
}

impl RecentProjectsList {
    pub fn add(&mut self, path: PathBuf, timestamp: String) {
        self.projects.retain(|p| p.path != path);
        self.projects.insert(
            0,
            RecentProject {
                path,
                last_opened: timestamp,
            },
        );
        self.projects.truncate(MAX_RECENT_PROJECTS);
    }

    pub fn remove(&mut self, path: &Path) {
        self.projects.retain(|p| p.path != path);
    }

    pub fn prune(&mut self, mut should_retain: impl FnMut(&Path) -> bool) {
        self.projects.retain(|p| should_retain(&p.path));
    }
}
