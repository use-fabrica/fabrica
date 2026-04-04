use std::fs;
use std::path::{Path, PathBuf};

use crate::recent_projects_list::RecentProject;
use crate::time_utils::now_iso;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RecentProjectsFile {
    recent_projects: Vec<RecentProject>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct RecentProjects {
    pub projects: Vec<RecentProject>,
    file_path: PathBuf,
}

impl Default for RecentProjects {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            file_path: PathBuf::new(),
        }
    }
}

impl RecentProjects {
    fn default_with_path(file_path: PathBuf) -> Self {
        Self {
            projects: Vec::new(),
            file_path,
        }
    }

    pub fn load(path: impl Into<PathBuf>) -> Self {
        let file_path = path.into();
        let json = match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(_) => return Self::default_with_path(file_path),
        };
        let data: serde_json::Value = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {}", file_path.display(), e);
                return Self::default_with_path(file_path);
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

    pub fn save(&self) -> std::io::Result<()> {
        let file_data = RecentProjectsFile {
            recent_projects: self.projects.clone(),
        };
        let json = serde_json::to_string_pretty(&file_data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = self.file_path.with_extension("json.tmp");
        fs::write(&tmp_path, &json)?;
        fs::rename(&tmp_path, &self.file_path)?;
        Ok(())
    }

    pub fn add(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        if !path.exists() || !path.is_dir() {
            return;
        }
        self.projects.retain(|p| p.path != path);
        self.projects.insert(
            0,
            RecentProject {
                path: path.clone(),
                last_opened: now_iso(),
            },
        );
        self.projects.truncate(5);
        let _ = self.save();
    }

    pub fn remove(&mut self, path: &Path) {
        self.projects.retain(|p| p.path != path);
        let _ = self.save();
    }

    pub fn prune(&mut self) {
        self.projects.retain(|p| p.path.exists() && p.path.is_dir());
        let _ = self.save();
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

        let rp = RecentProjects::load(&path);
        assert_eq!(rp.projects.len(), 1);
        assert_eq!(rp.projects[0].path, PathBuf::from("/tmp/test"));
        assert_eq!(rp.projects[0].last_opened, "2025-01-01T00:00:00Z");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing_file() {
        let path = std::env::temp_dir().join("fabrica_nonexistent_12345.json");
        let rp = RecentProjects::load(&path);
        assert!(rp.projects.is_empty());
    }

    #[test]
    fn test_load_corrupted_json() {
        let dir = std::env::temp_dir().join("fabrica_test_corrupted");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("corrupted.json");

        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"{broken").unwrap();
        drop(f);

        let rp = RecentProjects::load(&path);
        assert!(rp.projects.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("fabrica_test_roundtrip");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("roundtrip.json");

        let mut rp = RecentProjects::default_with_path(path.clone());
        rp.projects.push(RecentProject {
            path: PathBuf::from("/tmp/project1"),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.projects.push(RecentProject {
            path: PathBuf::from("/tmp/project2"),
            last_opened: "2025-02-01T00:00:00Z".to_string(),
        });
        rp.projects.push(RecentProject {
            path: PathBuf::from("/tmp/project3"),
            last_opened: "2025-03-01T00:00:00Z".to_string(),
        });

        rp.save().unwrap();

        let loaded = RecentProjects::load(&path);
        assert_eq!(loaded.projects.len(), 3);
        assert_eq!(loaded.projects[0].path, PathBuf::from("/tmp/project1"));
        assert_eq!(loaded.projects[1].path, PathBuf::from("/tmp/project2"));
        assert_eq!(loaded.projects[2].path, PathBuf::from("/tmp/project3"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_list_save_load() {
        let dir = std::env::temp_dir().join("fabrica_test_empty");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("empty.json");

        let rp = RecentProjects::default_with_path(path.clone());
        assert!(rp.projects.is_empty());

        rp.save().unwrap();

        let loaded = RecentProjects::load(&path);
        assert!(loaded.projects.is_empty());

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

        let mut rp = RecentProjects::default_with_path(file_path);
        rp.add(&temp_dir);

        assert_eq!(rp.projects.len(), 1);
        assert_eq!(rp.projects[0].path, temp_dir);
        assert!(!rp.projects[0].last_opened.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_add_existing_project_dedup() {
        let dir = std::env::temp_dir().join("fabrica_test_add_dedup");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let dir_a = dir.join("project_a");
        let dir_b = dir.join("project_b");
        let _ = fs::create_dir_all(&dir_a);
        let _ = fs::create_dir_all(&dir_b);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::default_with_path(file_path);
        rp.add(&dir_a);
        rp.add(&dir_b);

        assert_eq!(rp.projects.len(), 2);
        assert_eq!(rp.projects[0].path, dir_b);
        assert_eq!(rp.projects[1].path, dir_a);

        let ts_before = rp.projects[1].last_opened.clone();
        rp.add(&dir_a);

        assert_eq!(rp.projects.len(), 2);
        assert_eq!(rp.projects[0].path, dir_a);
        assert_eq!(rp.projects[1].path, dir_b);
        assert!(rp.projects[0].last_opened >= ts_before);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_truncation_at_five() {
        let dir = std::env::temp_dir().join("fabrica_test_truncation");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("recent.json");

        let mut rp = RecentProjects::default_with_path(file_path);
        let mut project_dirs = Vec::new();
        for i in 0..6 {
            let p = dir.join(format!("project_{}", i));
            let _ = fs::create_dir_all(&p);
            project_dirs.push(p);
        }

        for p in &project_dirs {
            rp.add(p);
        }

        assert_eq!(rp.projects.len(), 5);
        assert_eq!(rp.projects[0].path, project_dirs[5]);
        assert_eq!(rp.projects[1].path, project_dirs[4]);

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

        let mut rp = RecentProjects::default_with_path(file_path);
        rp.add(&dir_a);
        rp.add(&dir_b);
        rp.add(&dir_c);

        rp.remove(&dir_b);

        assert_eq!(rp.projects.len(), 2);
        assert_eq!(rp.projects[0].path, dir_c);
        assert_eq!(rp.projects[1].path, dir_a);

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

        let mut rp = RecentProjects::default_with_path(file_path);
        rp.projects.push(RecentProject {
            path: real_dir.clone(),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });
        rp.projects.push(RecentProject {
            path: fake_path.clone(),
            last_opened: "2025-01-01T00:00:00Z".to_string(),
        });

        assert_eq!(rp.projects.len(), 2);

        rp.prune();

        assert_eq!(rp.projects.len(), 1);
        assert_eq!(rp.projects[0].path, real_dir);

        let _ = fs::remove_dir_all(&dir);
    }
}
