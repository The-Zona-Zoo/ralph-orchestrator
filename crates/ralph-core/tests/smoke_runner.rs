//! Integration tests for the smoke test replay runner.

use ralph_core::testing::{list_fixtures, SmokeRunner, SmokeTestConfig, TerminationReason};
use std::path::PathBuf;

/// Returns the path to the test fixtures directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

// ─────────────────────────────────────────────────────────────────────────────
// Acceptance Criteria #6: Example Fixture Included
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_fixtures_directory_exists() {
    let dir = fixtures_dir();
    assert!(dir.exists(), "Fixtures directory should exist at {:?}", dir);
}

#[test]
fn test_basic_session_fixture_exists() {
    let fixture = fixtures_dir().join("basic_session.jsonl");
    assert!(
        fixture.exists(),
        "Basic session fixture should exist at {:?}",
        fixture
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Acceptance Criteria #7: Integration Test Validates Full Replay Flow
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_full_replay_flow_with_basic_session() {
    let fixture = fixtures_dir().join("basic_session.jsonl");

    let config = SmokeTestConfig::new(&fixture);
    let result = SmokeRunner::run(&config).expect("Should run fixture successfully");

    // Verify completion
    assert!(
        result.completed_successfully(),
        "Basic session should complete successfully"
    );
    assert_eq!(
        *result.termination_reason(),
        TerminationReason::Completed,
        "Should terminate with Completed (LOOP_COMPLETE detected)"
    );

    // Verify iterations (one per terminal write chunk)
    assert!(
        result.iterations_run() >= 2,
        "Should process at least 2 chunks (completion found in 3rd)"
    );

    // Verify event parsing
    // Fixture contains: build.task and build.done events
    assert!(
        result.event_count() >= 2,
        "Should parse at least 2 events from fixture, got {}",
        result.event_count()
    );

    // Verify output processing
    assert!(
        result.output_bytes() > 0,
        "Should have processed some output bytes"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Acceptance Criteria #6: Fixture Discovery
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_fixture_discovery() {
    let fixtures = list_fixtures(fixtures_dir()).expect("Should list fixtures");

    // Should find at least basic_session.jsonl
    assert!(
        !fixtures.is_empty(),
        "Should find at least one fixture in {:?}",
        fixtures_dir()
    );

    let fixture_names: Vec<_> = fixtures
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .collect();

    assert!(
        fixture_names.contains(&"basic_session.jsonl".to_string()),
        "Should discover basic_session.jsonl, found: {:?}",
        fixture_names
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Programmatic Fixture Loading
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_all_discovered_fixtures_are_valid() {
    let fixtures = list_fixtures(fixtures_dir()).expect("Should list fixtures");

    for fixture_path in fixtures {
        let config = SmokeTestConfig::new(&fixture_path);
        let result = SmokeRunner::run(&config);

        assert!(
            result.is_ok(),
            "Fixture {:?} should be valid and runnable: {:?}",
            fixture_path,
            result.err()
        );
    }
}
