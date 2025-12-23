use chrono::Utc;
use sqlx::{PgPool, query_as};

use crate::db::model::Group;


pub fn current_academic_year() -> i32 {
    use chrono::Datelike;
    let now = Utc::now();
    if now.month() >= 9 {
        now.year() as i32
    } else {
        (now.year() - 1) as i32
    }
}

pub async fn active_groups(pool: &PgPool) -> Result<Vec<Group>, sqlx::Error> {
    query_as!(
        Group,
        r#"
            select id, name, academic_year, created_at, updated_at from "group"
            where academic_year = ($1)
        "#,
        current_academic_year()
    )
    .fetch_all(pool)
    .await
}
