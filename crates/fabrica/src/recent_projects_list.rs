use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(crate) const MAX_RECENT_PROJECTS: usize = 5;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct RecentProject {
    pub path: PathBuf,
    pub last_opened: String,
}

/// Pure in-memory list of recent projects with NO filesystem access and NO clock dependency.
#[derive(Default)]
pub(crate) struct RecentProjectsList {
    pub projects: Vec<RecentProject>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_new_project() {
        let mut list = RecentProjectsList::default();
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:00:00Z".to_string());
        assert_eq!(list.projects.len(), 1);
        assert_eq!(list.projects[0].path, PathBuf::from("/tmp/a"));
        assert_eq!(list.projects[0].last_opened, "2025-01-01T00:00:00Z");
    }

    #[test]
    fn test_add_dedup_moves_to_top() {
        let mut list = RecentProjectsList::default();
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:00:00Z".to_string());
        list.add(PathBuf::from("/tmp/b"), "2025-01-01T00:00:00Z".to_string());
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:05:00Z".to_string());

        assert_eq!(list.projects.len(), 2);
        assert_eq!(list.projects[0].path, PathBuf::from("/tmp/a"));
        assert_eq!(list.projects[0].last_opened, "2025-01-01T00:05:00Z");
        assert_eq!(list.projects[1].path, PathBuf::from("/tmp/b"));
    }

    #[test]
    fn test_add_truncation_at_five() {
        let mut list = RecentProjectsList::default();
        for i in 0..6 {
            list.add(
                PathBuf::from(format!("/tmp/p{}", i)),
                format!("2025-01-01T00:00:{:02}Z", i),
            );
        }
        assert_eq!(list.projects.len(), 5);
        assert_eq!(list.projects.last().unwrap().path, PathBuf::from("/tmp/p1"));
    }

    #[test]
    fn test_add_truncation_exact_five() {
        let mut list = RecentProjectsList::default();
        for i in 0..5 {
            list.add(
                PathBuf::from(format!("/tmp/p{}", i)),
                format!("2025-01-01T00:00:{:02}Z", i),
            );
        }
        assert_eq!(list.projects.len(), 5);
    }

    #[test]
    fn test_remove_existing() {
        let mut list = RecentProjectsList::default();
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:00:00Z".to_string());
        list.add(PathBuf::from("/tmp/b"), "2025-01-01T00:00:00Z".to_string());
        list.remove(Path::new("/tmp/a"));

        assert_eq!(list.projects.len(), 1);
        assert_eq!(list.projects[0].path, PathBuf::from("/tmp/b"));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut list = RecentProjectsList::default();
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:00:00Z".to_string());
        list.remove(Path::new("/tmp/nonexistent"));
        assert_eq!(list.projects.len(), 1);
    }

    #[test]
    fn test_prune_retain_all() {
        let mut list = RecentProjectsList::default();
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:00:00Z".to_string());
        list.add(PathBuf::from("/tmp/b"), "2025-01-01T00:00:00Z".to_string());

        let mut pred = |_p: &Path| true;
        list.prune(&mut pred);

        assert_eq!(list.projects.len(), 2);
    }

    #[test]
    fn test_prune_remove_all() {
        let mut list = RecentProjectsList::default();
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:00:00Z".to_string());
        list.add(PathBuf::from("/tmp/b"), "2025-01-01T00:00:00Z".to_string());

        let mut pred = |_p: &Path| false;
        list.prune(&mut pred);

        assert_eq!(list.projects.len(), 0);
    }

    #[test]
    fn test_prune_selective() {
        let mut list = RecentProjectsList::default();
        list.add(PathBuf::from("/tmp/a"), "2025-01-01T00:00:00Z".to_string());
        list.add(PathBuf::from("/tmp/b"), "2025-01-01T00:00:00Z".to_string());
        list.add(PathBuf::from("/tmp/c"), "2025-01-01T00:00:00Z".to_string());

        list.prune(&mut |p: &Path| p == Path::new("/tmp/a") || p == Path::new("/tmp/c"));

        assert_eq!(list.projects.len(), 2);
        assert_eq!(list.projects[0].path, PathBuf::from("/tmp/c"));
        assert_eq!(list.projects[1].path, PathBuf::from("/tmp/a"));
    }

    #[test]
    fn test_empty_list_operations() {
        let mut list = RecentProjectsList::default();
        list.remove(Path::new("/tmp/nonexistent"));
        assert!(list.projects.is_empty());

        let mut pred = |_p: &Path| false;
        list.prune(&mut pred);
        assert!(list.projects.is_empty());
    }
}
