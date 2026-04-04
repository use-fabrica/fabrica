use std::fs;
use std::path::{Path, PathBuf};

use crate::time_utils::now_iso;
use serde::{Deserialize, Serialize};

pub(crate) const MAX_RECENT_PROJECTS: usize = 5;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct RecentProject {
    pub path: PathBuf,
    pub last_opened: String,
}

#[derive(Serialize, Deserialize)]
struct RecentProjectsFile {
    recent_projects: Vec<RecentProject>,
}

pub(crate) struct RecentProjects {
    projects: Vec<RecentProject>,
    file_path: PathBuf,
}

impl RecentProjects {
    /// Load recent projects from the default path (~/.fabrica/recent.json).
    /// Prunes non-existent entries. Never fails — returns empty on error.
    pub(crate) fn load() -> Self {
        let file_path = Self::data_dir().join("recent.json");
        Self::load_from_path(file_path)
    }

    /// Add a project to the list. Deduplicates, truncates to MAX_RECENT_PROJECTS, auto-saves.
    pub(crate) fn add(&mut self, path: &Path) {
        if !path.exists() || !path.is_dir() {
            return;
        }
        self.projects.retain(|p| p.path != path);
        self.projects.insert(
            0,
            RecentProject {
                path: path.to_path_buf(),
                last_opened: now_iso(),
            },
        );
        self.projects.truncate(MAX_RECENT_PROJECTS);
        let _ = self.save();
    }

    /// Remove a project from the list. Auto-saves.
    pub(crate) fn remove(&mut self, path: &Path) {
        self.projects.retain(|p| p.path != path);
        let _ = self.save();
    }

    /// Remove projects whose paths no longer exist. Auto-saves.
    pub(crate) fn prune(&mut self) {
        self.projects.retain(|p| p.path.exists() && p.path.is_dir());
        let _ = self.save();
    }

    pub(crate) fn list(&self) -> &[RecentProject] {
        &self.projects
    }

    fn save(&self) -> std::io::Result<()> {
        let file_data = RecentProjectsFile {
            recent_projects: self.projects.clone(),
        };
        let json = serde_json::to_string_pretty(&file_data).map_err(std::io::Error::other)?;

        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = self.file_path.with_extension("json.tmp");
        fs::write(&tmp_path, &json)?;
        fs::rename(&tmp_path, &self.file_path)?;
        Ok(())
    }

    fn data_dir() -> PathBuf {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".fabrica"))
            .unwrap_or_else(|_| PathBuf::from(".fabrica"))
    }

    fn load_from_path(file_path: PathBuf) -> Self {
        let json = match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(_) => return Self::empty_with_path(file_path),
        };
        let data: serde_json::Value = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {}", file_path.display(), e);
                return Self::empty_with_path(file_path);
            }
        };
        let projects: Vec<RecentProject> = match data.get("recent_projects").cloned() {
            Some(v) => match serde_json::from_value(v) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Warning: failed to parse recent_projects: {}", e);
                    Vec::new()
                }
            },
            None => Vec::new(),
        };
        Self {
            projects,
            file_path,
        }
    }

    fn empty_with_path(file_path: PathBuf) -> Self {
        Self {
            projects: Vec::new(),
            file_path,
        }
    }
}

#[cfg(test)]
impl RecentProjects {
    /// Test-only: load from a custom path.
    pub(crate) fn load_from(path: impl Into<PathBuf>) -> Self {
        Self::load_from_path(path.into())
    }

    /// Test-only: push a project directly for test setup.
    pub(crate) fn push_project(&mut self, project: RecentProject) {
        self.projects.push(project);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_valid_json() {
        let dir = std::env::temp_dir().join("fabrica_test_load_valid");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("valid.json");

        let mut f = fs::File::create(&path).unwrap();
        f.write_all(
            br#"{"recent_projects":[{"path":"/tmp/test","last_opened":"2025-01-01T00:00:00Z"}]}"#,
        )
        .unwrap();
        drop(f);

        let rp = RecentProjects::load_from(&path);
        assert_eq!(rp.list().len(), 1);
        assert_eq!(rp.list()[0].path, PathBuf::from("/tmp/test"));
        assert_eq!(rp.list()[0].last_opened, "2025-01-01T00:00:00Z");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing_file() {
        let path = std::env::temp_dir().join("fabrica_nonexistent_12345.json");
        let rp = RecentProjects::load_from(&path);
        assert!(rp.list().is_empty());
    }

    #[test]
    fn test_load_corrupted_json() {
        let dir = std::env::temp_dir().join("fabrica_test_corrupted");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("corrupted.json");

        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"{broken").unwrap();
        drop(f);

        let rp = RecentProjects::load_from(&path);
        assert!(rp.list().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("fabrica_test_roundtrip");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("roundtrip.json");

        let mut rp = RecentProjects::load_from(path.clone());
        rp.push_project(RecentProject {
            path: PathBuf::from("/tmp/project1"),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.push_project(RecentProject {
            path: PathBuf::from("/tmp/project2"),
            last_opened: "2025-02-01T00:00:00Z".to_string(),
        });
        rp.push_project(RecentProject {
            path: PathBuf::from("/tmp/project3"),
            last_opened: "2025-03-01T00:00:00Z".to_string(),
        });

        rp.save().unwrap();

        let loaded = RecentProjects::load_from(&path);
        assert_eq!(loaded.list().len(), 3);
        assert_eq!(loaded.list()[0].path, PathBuf::from("/tmp/project1"));
        assert_eq!(loaded.list()[1].path, PathBuf::from("/tmp/project2"));
        assert_eq!(loaded.list()[2].path, PathBuf::from("/tmp/project3"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_list_save_load() {
        let dir = std::env::temp_dir().join("fabrica_test_empty");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("empty.json");

        let rp = RecentProjects::load_from(path.clone());
        assert!(rp.list().is_empty());

        rp.save().unwrap();

        let loaded = RecentProjects::load_from(&path);
        assert!(loaded.list().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_new_project() {
        let dir = std::env::temp_dir().join("fabrica_test_add_new");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let temp_dir = dir.join("my_project");
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.add(&temp_dir);

        assert_eq!(rp.list().len(), 1);
        assert_eq!(rp.list()[0].path, temp_dir);
        assert!(!rp.list()[0].last_opened.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_project() {
        let dir = std::env::temp_dir().join("fabrica_test_remove");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("project_a");
        let dir_b = dir.join("project_b");
        let dir_c = dir.join("project_c");
        let _ = fs::create_dir_all(&dir_a);
        let _ = fs::create_dir_all(&dir_b);
        let _ = fs::create_dir_all(&dir_c);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.add(&dir_a);
        rp.add(&dir_b);
        rp.add(&dir_c);

        rp.remove(&dir_b);

        assert_eq!(rp.list().len(), 2);
        assert_eq!(rp.list()[0].path, dir_c);
        assert_eq!(rp.list()[1].path, dir_a);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_prune_nonexistent_paths() {
        let dir = std::env::temp_dir().join("fabrica_test_prune");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let real_dir = dir.join("real_project");
        let _ = fs::create_dir_all(&real_dir);
        let fake_path = dir.join("does_not_exist_at_all");
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.push_project(RecentProject {
            path: real_dir.clone(),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.push_project(RecentProject {
            path: fake_path.clone(),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });

        assert_eq!(rp.list().len(), 2);

        rp.prune();

        assert_eq!(rp.list().len(), 1);
        assert_eq!(rp.list()[0].path, real_dir);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_persists_to_disk() {
        let dir = std::env::temp_dir().join("fabrica_test_add_persist");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let real_dir = dir.join("project_x");
        let _ = fs::create_dir_all(&real_dir);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path.clone());
        rp.add(&real_dir);

        let loaded = RecentProjects::load_from(&file_path);
        assert_eq!(loaded.list().len(), 1);
        assert_eq!(loaded.list()[0].path, real_dir);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_persists_to_disk() {
        let dir = std::env::temp_dir().join("fabrica_test_remove_persist");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("project_a");
        let dir_b = dir.join("project_b");
        let _ = fs::create_dir_all(&dir_a);
        let _ = fs::create_dir_all(&dir_b);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path.clone());
        rp.add(&dir_a);
        rp.add(&dir_b);
        rp.remove(&dir_a);

        let loaded = RecentProjects::load_from(&file_path);
        assert_eq!(loaded.list().len(), 1);
        assert_eq!(loaded.list()[0].path, dir_b);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_prune_persists_to_disk() {
        let dir = std::env::temp_dir().join("fabrica_test_prune_persist");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let real_dir = dir.join("real_project");
        let _ = fs::create_dir_all(&real_dir);
        let fake_path = dir.join("does_not_exist");
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path.clone());
        rp.push_project(RecentProject {
            path: real_dir.clone(),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.push_project(RecentProject {
            path: fake_path,
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.save().unwrap();
        rp.prune();

        let loaded = RecentProjects::load_from(&file_path);
        assert_eq!(loaded.list().len(), 1);
        assert_eq!(loaded.list()[0].path, real_dir);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_rejects_nonexistent_path() {
        let dir = std::env::temp_dir().join("fabrica_test_reject_nonexistent");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let fake_path = dir.join("no_such_dir");
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path.clone());
        rp.add(&fake_path);

        assert!(rp.list().is_empty());
        assert!(!file_path.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_dedup_moves_to_top() {
        let dir = std::env::temp_dir().join("fabrica_test_dedup");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("a");
        let dir_b = dir.join("b");
        let _ = fs::create_dir_all(&dir_a);
        let _ = fs::create_dir_all(&dir_b);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.add(&dir_a);
        rp.add(&dir_b);
        rp.add(&dir_a);

        assert_eq!(rp.list().len(), 2);
        assert_eq!(rp.list()[0].path, dir_a);
        assert_eq!(rp.list()[1].path, dir_b);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_truncation_at_five() {
        let dir = std::env::temp_dir().join("fabrica_test_truncation");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        for i in 0..6 {
            let _ = fs::create_dir_all(dir.join(format!("p{}", i)));
        }
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        for i in 0..6 {
            rp.add(&dir.join(format!("p{}", i)));
        }
        assert_eq!(rp.list().len(), 5);
        assert_eq!(rp.list().last().unwrap().path, dir.join("p1"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_truncation_exact_five() {
        let dir = std::env::temp_dir().join("fabrica_test_truncation_exact");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        for i in 0..5 {
            let _ = fs::create_dir_all(dir.join(format!("p{}", i)));
        }
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        for i in 0..5 {
            rp.add(&dir.join(format!("p{}", i)));
        }
        assert_eq!(rp.list().len(), 5);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_existing() {
        let dir = std::env::temp_dir().join("fabrica_test_remove_existing");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("a");
        let dir_b = dir.join("b");
        let _ = fs::create_dir_all(&dir_a);
        let _ = fs::create_dir_all(&dir_b);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.add(&dir_a);
        rp.add(&dir_b);
        rp.remove(&dir_a);

        assert_eq!(rp.list().len(), 1);
        assert_eq!(rp.list()[0].path, dir_b);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_nonexistent() {
        let dir = std::env::temp_dir().join("fabrica_test_remove_nonexistent");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("a");
        let _ = fs::create_dir_all(&dir_a);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.add(&dir_a);
        rp.remove(Path::new("/tmp/nonexistent"));
        assert_eq!(rp.list().len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_prune_retain_all() {
        let dir = std::env::temp_dir().join("fabrica_test_prune_retain");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("a");
        let dir_b = dir.join("b");
        let _ = fs::create_dir_all(&dir_a);
        let _ = fs::create_dir_all(&dir_b);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.add(&dir_a);
        rp.add(&dir_b);
        rp.prune();

        assert_eq!(rp.list().len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_prune_remove_all() {
        let dir = std::env::temp_dir().join("fabrica_test_prune_remove_all");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let fake_a = dir.join("fake_a");
        let fake_b = dir.join("fake_b");
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.push_project(RecentProject {
            path: fake_a,
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.push_project(RecentProject {
            path: fake_b,
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.prune();

        assert_eq!(rp.list().len(), 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_prune_selective() {
        let dir = std::env::temp_dir().join("fabrica_test_prune_selective");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("a");
        let dir_c = dir.join("c");
        let _ = fs::create_dir_all(&dir_a);
        let _ = fs::create_dir_all(&dir_c);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.push_project(RecentProject {
            path: dir_a.clone(),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.push_project(RecentProject {
            path: dir.join("b"),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.push_project(RecentProject {
            path: dir_c.clone(),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.prune();

        assert_eq!(rp.list().len(), 2);
        assert_eq!(rp.list()[0].path, dir_a);
        assert_eq!(rp.list()[1].path, dir_c);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_list_operations() {
        let dir = std::env::temp_dir().join("fabrica_test_empty_ops");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::load_from(file_path);
        rp.remove(Path::new("/tmp/nonexistent"));
        assert!(rp.list().is_empty());

        rp.prune();
        assert!(rp.list().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}
