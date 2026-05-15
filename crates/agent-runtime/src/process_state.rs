use agent_protocol::McpServerId;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct RuntimeProcessStateFile {
    pub records: Vec<RuntimeProcessRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct RuntimeProcessRecord {
    pub server_id: McpServerId,
    pub pid: Option<u32>,
    pub command: Option<String>,
    pub started_at_unix: u64,
    pub state: RuntimeProcessState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProcessState {
    Running,
    Stopped,
}

impl RuntimeProcessStateFile {
    pub fn record_start(&mut self, record: RuntimeProcessRecord) {
        self.records
            .retain(|existing| existing.server_id != record.server_id);
        self.records.push(record);
        self.records.sort_by(|left, right| {
            left.server_id
                .as_str()
                .cmp(right.server_id.as_str())
                .then(left.pid.cmp(&right.pid))
        });
    }

    pub fn mark_stopped(&mut self, server_id: &McpServerId) -> bool {
        let mut changed = false;
        for record in &mut self.records {
            if &record.server_id == server_id && record.state != RuntimeProcessState::Stopped {
                record.state = RuntimeProcessState::Stopped;
                changed = true;
            }
        }
        changed
    }

    pub fn prune_stopped(&mut self) -> Vec<RuntimeProcessRecord> {
        let mut retained = Vec::new();
        let mut removed = Vec::new();
        for record in std::mem::take(&mut self.records) {
            if record.state == RuntimeProcessState::Stopped {
                removed.push(record);
            } else {
                retained.push(record);
            }
        }
        self.records = retained;
        removed
    }

    pub fn stale_running_records(&self, live_pids: &BTreeSet<u32>) -> Vec<RuntimeProcessRecord> {
        self.records
            .iter()
            .filter(|record| {
                record.state == RuntimeProcessState::Running
                    && record.pid.is_some_and(|pid| !live_pids.contains(&pid))
            })
            .cloned()
            .collect()
    }

    pub fn mark_missing_pids_stopped(
        &mut self,
        live_pids: &BTreeSet<u32>,
    ) -> Vec<RuntimeProcessRecord> {
        let stale_by_server: BTreeMap<McpServerId, RuntimeProcessRecord> = self
            .stale_running_records(live_pids)
            .into_iter()
            .map(|record| (record.server_id.clone(), record))
            .collect();
        for record in &mut self.records {
            if stale_by_server.contains_key(&record.server_id) {
                record.state = RuntimeProcessState::Stopped;
            }
        }
        stale_by_server.into_values().collect()
    }
}

pub fn load_process_state(
    path: impl AsRef<Path>,
) -> Result<RuntimeProcessStateFile, ProcessStateError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(RuntimeProcessStateFile::default());
    }
    let raw = std::fs::read_to_string(path).map_err(ProcessStateError::Io)?;
    serde_json::from_str(&raw).map_err(ProcessStateError::Serde)
}

pub fn write_process_state(
    path: impl AsRef<Path>,
    state: &RuntimeProcessStateFile,
) -> Result<(), ProcessStateError> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(ProcessStateError::Io)?;
    }
    let raw = serde_json::to_string_pretty(state).map_err(ProcessStateError::Serde)?;
    std::fs::write(path, raw).map_err(ProcessStateError::Io)
}

#[derive(Debug, Error)]
pub enum ProcessStateError {
    #[error("failed to read or write MCP runtime process state: {0}")]
    Io(std::io::Error),
    #[error("failed to parse MCP runtime process state: {0}")]
    Serde(serde_json::Error),
}
