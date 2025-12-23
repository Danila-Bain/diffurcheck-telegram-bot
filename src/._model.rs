use time::{Duration, Time};

pub struct Group {
    pub id: i64,
    pub name: String,
    pub year: i32
}

pub struct User {
    pub id: i64,
    pub name: String,
    pub group: Group,
    pub pw_hash: Vec<u8>,
    pub progress: Vec<AssignmentEntry>,
}

pub struct AssignmentEntry {
    pub id: i64,
    pub problems: Vec<Problem>,
    pub score: f64,
    pub time_start: Time,
    pub duration: Duration,
}

pub struct Problem {
    pub problem: String,
    pub true_answer: String,
    pub given_answer: Option<String>,
    pub is_answer_correct: bool,
}
