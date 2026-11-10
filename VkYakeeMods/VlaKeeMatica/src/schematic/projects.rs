use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::litematic::LitematicaSchematic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub version: u32,
    pub timestamp: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct SchematicProject {
    pub name: String,
    pub schematic: LitematicaSchematic,
    pub versions: Vec<ProjectVersion>,
    pub current_version: u32,
    pub project_file: Option<PathBuf>,
    pub is_modified: bool,
}

impl SchematicProject {
    pub fn new(name: impl Into<String>, schematic: LitematicaSchematic) -> Self {
        let name = name.into();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let initial_version = ProjectVersion {
            version: 1,
            timestamp: now,
            name: "Initial".to_string(),
            description: "Initial project creation".to_string(),
        };

        Self {
            name,
            schematic,
            versions: vec![initial_version],
            current_version: 1,
            project_file: None,
            is_modified: false,
        }
    }

    pub fn commit_version(&mut self, version_name: impl Into<String>, description: impl Into<String>) -> u32 {
        self.current_version += 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        self.versions.push(ProjectVersion {
            version: self.current_version,
            timestamp: now,
            name: version_name.into(),
            description: description.into(),
        });
        self.is_modified = false;
        self.current_version
    }
}

#[derive(Debug, Default)]
pub struct SchematicProjectsManager {
    active_project: Option<SchematicProject>,
}

impl SchematicProjectsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_active_project(&mut self, project: SchematicProject) {
        self.active_project = Some(project);
    }

    pub fn get_active_project(&self) -> Option<&SchematicProject> {
        self.active_project.as_ref()
    }

    pub fn get_active_project_mut(&mut self) -> Option<&mut SchematicProject> {
        self.active_project.as_mut()
    }

    pub fn has_project_open(&self) -> bool {
        self.active_project.is_some()
    }

    pub fn close_project(&mut self) {
        self.active_project = None;
    }

    pub fn list_versions(&self) -> Vec<String> {
        if let Some(ref project) = self.active_project {
            project.versions.iter()
                .map(|v| format!("v{} - {} ({})", v.version, v.name, v.description))
                .collect()
        } else {
            Vec::new()
        }
    }
}
