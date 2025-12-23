use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;
use crate::db::Group;

#[derive(Debug, Clone)]
pub struct Assignment {
    pub id: Uuid,
    pub title: String,
    pub topics: String,
    pub rules: String,
    pub deadline: DateTime<Utc>,
    pub duration: Duration,
    pub group: Vec<Group>,
}
