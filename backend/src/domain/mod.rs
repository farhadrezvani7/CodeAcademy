//! Pure business rules. Nothing in this module touches the database or HTTP,
//! which keeps every rule unit-testable in isolation.

pub mod achievements;
pub mod advisor;
pub mod checkin;
pub mod course_status;
pub mod fa;
pub mod gpa;
pub mod jalali;
pub mod mission;
pub mod progression;
pub mod ranking;
pub mod streak;
pub mod templates;
pub mod xp;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    Locked,
    Enrolled,
    InProgress,
    Done,
}

impl EnrollmentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Locked => "locked",
            Self::Enrolled => "enrolled",
            Self::InProgress => "in_progress",
            Self::Done => "done",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "locked" => Self::Locked,
            "enrolled" => Self::Enrolled,
            "in_progress" => Self::InProgress,
            "done" => Self::Done,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourseKind {
    Course,
    Project,
}

impl CourseKind {
    pub fn parse(s: &str) -> Self {
        if s == "project" {
            Self::Project
        } else {
            Self::Course
        }
    }
}

/// Skill-tree node state, derived from the enrollment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    Done,
    InProgress,
    Locked,
}

impl NodeState {
    /// A finished course with a failing grade still needs work, so it is
    /// "in progress" rather than done.
    pub fn of(status: EnrollmentStatus, passed: bool) -> Self {
        match status {
            EnrollmentStatus::Done if passed => Self::Done,
            EnrollmentStatus::Done | EnrollmentStatus::Enrolled | EnrollmentStatus::InProgress => Self::InProgress,
            EnrollmentStatus::Locked => Self::Locked,
        }
    }
}

/// Percentage 0..=100, rounded down, safe for a zero total.
pub fn percent(part: i64, total: i64) -> i32 {
    if total <= 0 {
        0
    } else {
        ((part.clamp(0, total) * 100) / total) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_state_maps_enrollment() {
        assert_eq!(NodeState::of(EnrollmentStatus::Done, true), NodeState::Done);
        assert_eq!(NodeState::of(EnrollmentStatus::Done, false), NodeState::InProgress);
        assert_eq!(NodeState::of(EnrollmentStatus::Enrolled, false), NodeState::InProgress);
        assert_eq!(NodeState::of(EnrollmentStatus::InProgress, false), NodeState::InProgress);
        assert_eq!(NodeState::of(EnrollmentStatus::Locked, false), NodeState::Locked);
    }

    #[test]
    fn status_roundtrip() {
        for s in [EnrollmentStatus::Locked, EnrollmentStatus::Enrolled, EnrollmentStatus::InProgress, EnrollmentStatus::Done] {
            assert_eq!(EnrollmentStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(EnrollmentStatus::parse("bogus"), None);
    }

    #[test]
    fn percent_is_safe() {
        assert_eq!(percent(1, 0), 0);
        assert_eq!(percent(3, 4), 75);
        assert_eq!(percent(9, 4), 100);
    }
}
