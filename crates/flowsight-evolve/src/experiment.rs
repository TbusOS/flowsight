//! Experiment Tracking System
//!
//! Inspired by autoresearch's keep/discard loop:
//! ```text
//! experiment start "optimize-clk-kb"
//!     → create git branch, snapshot baseline AQS
//! experiment run
//!     → make change, measure AQS, keep/discard
//! experiment log
//!     → view TSV history
//! experiment best
//!     → show best result
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Where experiment data is stored
const EXPERIMENT_DIR: &str = ".flowsight-experiments";

/// TSV log filename
const LOG_FILE: &str = "evolution.tsv";

/// A single experiment entry (one row in evolution.tsv)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentEntry {
    /// Round number (1-indexed)
    pub round: usize,
    /// Git commit hash (None if discarded)
    pub commit: Option<String>,
    /// AQS before this round
    pub aqs_before: f64,
    /// AQS after this round
    pub aqs_after: f64,
    /// Delta (aqs_after - aqs_before)
    pub delta: f64,
    /// Status: keep or discard
    pub status: ExperimentStatus,
    /// Human-readable description
    pub description: String,
}

/// Status of an experiment round
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Keep,
    Discard,
}

impl fmt::Display for ExperimentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExperimentStatus::Keep => write!(f, "keep"),
            ExperimentStatus::Discard => write!(f, "discard"),
        }
    }
}

/// Experiment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMeta {
    /// Experiment name
    pub name: String,
    /// Git branch created for this experiment
    pub branch: String,
    /// Baseline AQS (measured at start)
    pub baseline_aqs: f64,
    /// Target directory for AQS measurement
    pub target_dir: String,
    /// File pattern
    pub pattern: String,
    /// Budget in seconds for each measurement
    pub budget_secs: f64,
    /// Total rounds completed
    pub rounds: usize,
}

/// Experiment tracker manages the experiment lifecycle
pub struct ExperimentTracker {
    /// Root directory for experiment data
    root: PathBuf,
}

impl ExperimentTracker {
    /// Create a tracker rooted at the given directory
    pub fn new(project_root: &Path) -> Self {
        Self {
            root: project_root.join(EXPERIMENT_DIR),
        }
    }

    /// Get path to the experiment directory
    pub fn experiment_dir(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    /// Get path to the metadata file
    fn meta_path(&self, name: &str) -> PathBuf {
        self.experiment_dir(name).join("meta.json")
    }

    /// Get path to the evolution log
    fn log_path(&self, name: &str) -> PathBuf {
        self.experiment_dir(name).join(LOG_FILE)
    }

    /// List all experiments
    pub fn list(&self) -> Vec<String> {
        if !self.root.exists() {
            return Vec::new();
        }
        fs::read_dir(&self.root)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Create a new experiment
    pub fn create(
        &self,
        name: &str,
        baseline_aqs: f64,
        target_dir: &str,
        pattern: &str,
        budget_secs: f64,
    ) -> Result<ExperimentMeta, String> {
        let dir = self.experiment_dir(name);
        if dir.exists() {
            return Err(format!("Experiment '{}' already exists", name));
        }

        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create experiment dir: {}", e))?;

        let branch = format!("experiment/{}", name);
        let meta = ExperimentMeta {
            name: name.to_string(),
            branch,
            baseline_aqs,
            target_dir: target_dir.to_string(),
            pattern: pattern.to_string(),
            budget_secs,
            rounds: 0,
        };

        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize meta: {}", e))?;
        fs::write(self.meta_path(name), meta_json)
            .map_err(|e| format!("Failed to write meta: {}", e))?;

        // Write TSV header
        fs::write(
            self.log_path(name),
            "round\tcommit\taqs_before\taqs_after\tdelta\tstatus\tdescription\n",
        )
        .map_err(|e| format!("Failed to write log header: {}", e))?;

        Ok(meta)
    }

    /// Load experiment metadata
    pub fn load_meta(&self, name: &str) -> Result<ExperimentMeta, String> {
        let path = self.meta_path(name);
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read meta: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse meta: {}", e))
    }

    /// Record a round result
    pub fn record_round(
        &self,
        name: &str,
        entry: &ExperimentEntry,
    ) -> Result<(), String> {
        // Append to TSV log
        let log_line = format!(
            "{}\t{}\t{:.4}\t{:.4}\t{:+.4}\t{}\t{}\n",
            entry.round,
            entry.commit.as_deref().unwrap_or("(none)"),
            entry.aqs_before,
            entry.aqs_after,
            entry.delta,
            entry.status,
            entry.description,
        );

        let log_path = self.log_path(name);
        let mut content =
            fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log: {}", e))?;
        content.push_str(&log_line);
        fs::write(&log_path, content).map_err(|e| format!("Failed to write log: {}", e))?;

        // Update meta
        let mut meta = self.load_meta(name)?;
        meta.rounds = entry.round;
        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize meta: {}", e))?;
        fs::write(self.meta_path(name), meta_json)
            .map_err(|e| format!("Failed to update meta: {}", e))?;

        Ok(())
    }

    /// Read all log entries for an experiment
    pub fn read_log(&self, name: &str) -> Result<Vec<ExperimentEntry>, String> {
        let log_path = self.log_path(name);
        let content =
            fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log: {}", e))?;

        let mut entries = Vec::new();
        for line in content.lines().skip(1) {
            // skip header
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 7 {
                continue;
            }

            let commit = if parts[1] == "(none)" {
                None
            } else {
                Some(parts[1].to_string())
            };

            let status = match parts[5] {
                "keep" => ExperimentStatus::Keep,
                _ => ExperimentStatus::Discard,
            };

            entries.push(ExperimentEntry {
                round: parts[0].parse().unwrap_or(0),
                commit,
                aqs_before: parts[2].parse().unwrap_or(0.0),
                aqs_after: parts[3].parse().unwrap_or(0.0),
                delta: parts[4].parse().unwrap_or(0.0),
                status,
                description: parts[6].to_string(),
            });
        }

        Ok(entries)
    }

    /// Find the best round (highest aqs_after among kept entries)
    pub fn best_round(&self, name: &str) -> Result<Option<ExperimentEntry>, String> {
        let entries = self.read_log(name)?;
        Ok(entries
            .into_iter()
            .filter(|e| e.status == ExperimentStatus::Keep)
            .max_by(|a, b| a.aqs_after.partial_cmp(&b.aqs_after).unwrap()))
    }

    /// Delete an experiment
    pub fn delete(&self, name: &str) -> Result<(), String> {
        let dir = self.experiment_dir(name);
        if dir.exists() {
            fs::remove_dir_all(&dir)
                .map_err(|e| format!("Failed to delete experiment: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_tracker() -> (ExperimentTracker, PathBuf) {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = env::temp_dir().join(format!(
            "flowsight-exp-test-{}-{}",
            std::process::id(),
            id
        ));
        // Clean up any leftover from previous runs
        let _ = fs::remove_dir_all(&dir);
        let tracker = ExperimentTracker::new(&dir);
        (tracker, dir)
    }

    #[test]
    fn test_create_and_load() {
        let (tracker, dir) = temp_tracker();
        let meta = tracker
            .create("test-exp", 0.75, "/tmp/test", "*.c", 30.0)
            .unwrap();

        assert_eq!(meta.name, "test-exp");
        assert_eq!(meta.branch, "experiment/test-exp");
        assert!((meta.baseline_aqs - 0.75).abs() < f64::EPSILON);
        assert_eq!(meta.rounds, 0);

        let loaded = tracker.load_meta("test-exp").unwrap();
        assert_eq!(loaded.name, "test-exp");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_record_and_read_log() {
        let (tracker, dir) = temp_tracker();
        tracker
            .create("test-log", 0.70, "/tmp", "*.c", 30.0)
            .unwrap();

        let entry1 = ExperimentEntry {
            round: 1,
            commit: Some("abc123".to_string()),
            aqs_before: 0.70,
            aqs_after: 0.73,
            delta: 0.03,
            status: ExperimentStatus::Keep,
            description: "added clk APIs".to_string(),
        };
        tracker.record_round("test-log", &entry1).unwrap();

        let entry2 = ExperimentEntry {
            round: 2,
            commit: None,
            aqs_before: 0.73,
            aqs_after: 0.71,
            delta: -0.02,
            status: ExperimentStatus::Discard,
            description: "tried platform bus expansion".to_string(),
        };
        tracker.record_round("test-log", &entry2).unwrap();

        let entries = tracker.read_log("test-log").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].status, ExperimentStatus::Keep);
        assert_eq!(entries[1].status, ExperimentStatus::Discard);

        let meta = tracker.load_meta("test-log").unwrap();
        assert_eq!(meta.rounds, 2);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_best_round() {
        let (tracker, dir) = temp_tracker();
        tracker
            .create("test-best", 0.60, "/tmp", "*.c", 30.0)
            .unwrap();

        for (i, (aqs_after, status)) in [
            (0.65, ExperimentStatus::Keep),
            (0.63, ExperimentStatus::Discard),
            (0.70, ExperimentStatus::Keep),
            (0.68, ExperimentStatus::Keep),
        ]
        .iter()
        .enumerate()
        {
            let entry = ExperimentEntry {
                round: i + 1,
                commit: if *status == ExperimentStatus::Keep {
                    Some(format!("commit{}", i))
                } else {
                    None
                },
                aqs_before: 0.60,
                aqs_after: *aqs_after,
                delta: aqs_after - 0.60,
                status: *status,
                description: format!("round {}", i + 1),
            };
            tracker.record_round("test-best", &entry).unwrap();
        }

        let best = tracker.best_round("test-best").unwrap().unwrap();
        assert_eq!(best.round, 3);
        assert!((best.aqs_after - 0.70).abs() < f64::EPSILON);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_list_experiments() {
        let (tracker, dir) = temp_tracker();
        tracker.create("exp-a", 0.5, "/tmp", "*.c", 10.0).unwrap();
        tracker.create("exp-b", 0.6, "/tmp", "*.c", 10.0).unwrap();

        let mut list = tracker.list();
        list.sort();
        assert_eq!(list, vec!["exp-a", "exp-b"]);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_delete_experiment() {
        let (tracker, dir) = temp_tracker();
        tracker.create("exp-del", 0.5, "/tmp", "*.c", 10.0).unwrap();
        assert!(tracker.experiment_dir("exp-del").exists());

        tracker.delete("exp-del").unwrap();
        assert!(!tracker.experiment_dir("exp-del").exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_duplicate_create_fails() {
        let (tracker, dir) = temp_tracker();
        tracker.create("dup", 0.5, "/tmp", "*.c", 10.0).unwrap();
        let result = tracker.create("dup", 0.5, "/tmp", "*.c", 10.0);
        assert!(result.is_err());

        fs::remove_dir_all(&dir).ok();
    }
}
