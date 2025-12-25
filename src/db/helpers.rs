use chrono::Utc;

pub fn current_academic_year() -> i32 {
    use chrono::Datelike;
    let now = Utc::now();
    if now.month() >= 9 {
        now.year() as i32
    } else {
        (now.year() - 1) as i32
    }
}
