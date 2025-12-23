use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub academic_year: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Student {
    pub id: Uuid,
    pub group_id: Uuid,
    pub telegram_id: i64,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>
}


#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub title: String,
    pub generator: String,
    pub solutions: Option<Vec<u8>>,
    pub graded_solutions: Option<Vec<u8>>,
    pub duration: Duration,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>

}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Variant {
    pub id: Uuid,
    pub variant_no: i32,
    pub assignemnt_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub student_id: Uuid,
    pub variant_id: Uuid,
    pub data: Vec<u8>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>
}
