use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Subscribe {
        job_id: String,
    },
    Unsubscribe {
        job_id: String,
    },
    Progress {
        job_id: String,
        file: String,
        percent: f32,
    },
    Complete {
        job_id: String,
    },
    Error {
        job_id: String,
        message: String,
    },
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub job_id: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub status: super::job::JobStatus,
    pub files: Vec<super::job::ConversionFile>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobListResponse {
    pub jobs: Vec<JobResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryEntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub name: String,
    pub path: String,
    pub kind: LibraryEntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryBrowseResponse {
    pub root_path: String,
    pub current_path: String,
    pub parent_path: Option<String>,
    pub entries: Vec<LibraryEntry>,
    pub directory_count: usize,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryConvertRequest {
    pub paths: Vec<String>,
}

impl WsMessage {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
