use super::*;

fn record(server_id: &str, pid: Option<u32>, state: RuntimeProcessState) -> RuntimeProcessRecord {
    RuntimeProcessRecord {
        server_id: McpServerId::new(server_id),
        pid,
        command: Some("node fixture.js".to_string()),
        started_at_unix: 42,
        state,
    }
}

#[test]
fn records_start_by_server_without_duplicate_running_rows() {
    let mut state = RuntimeProcessStateFile::default();
    state.record_start(record("b", Some(2), RuntimeProcessState::Running));
    state.record_start(record("a", Some(1), RuntimeProcessState::Running));
    state.record_start(record("a", Some(3), RuntimeProcessState::Running));

    assert_eq!(state.records.len(), 2);
    assert_eq!(state.records[0].server_id.as_str(), "a");
    assert_eq!(state.records[0].pid, Some(3));
}

#[test]
fn detects_and_marks_running_records_with_missing_pids() {
    let mut state = RuntimeProcessStateFile {
        records: vec![
            record("alive", Some(10), RuntimeProcessState::Running),
            record("stale", Some(20), RuntimeProcessState::Running),
            record("unknown-pid", None, RuntimeProcessState::Running),
            record("stopped", Some(30), RuntimeProcessState::Stopped),
        ],
    };
    let live_pids = BTreeSet::from([10]);

    let stale = state.stale_running_records(&live_pids);
    assert_eq!(
        stale
            .iter()
            .map(|record| record.server_id.as_str())
            .collect::<Vec<_>>(),
        vec!["stale"]
    );

    let marked = state.mark_missing_pids_stopped(&live_pids);
    assert_eq!(marked.len(), 1);
    assert_eq!(state.records[1].state, RuntimeProcessState::Stopped);
    assert_eq!(state.records[2].state, RuntimeProcessState::Running);
}

#[test]
fn round_trips_process_state_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("runtime/processes.json");
    let mut state = RuntimeProcessStateFile::default();
    state.record_start(record("fixture", Some(1234), RuntimeProcessState::Running));

    write_process_state(&path, &state).expect("write state");
    let loaded = load_process_state(&path).expect("load state");

    assert_eq!(loaded, state);
}
