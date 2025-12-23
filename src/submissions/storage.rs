use uuid::Uuid;
use teloxide::types::Message;
use std::path::PathBuf;

pub async fn store_submission(
    session_id: Uuid,
    _msg: &Message,
) -> anyhow::Result<()> {
    let _base = PathBuf::from("data")
        .join(session_id.to_string());

    // save documents, photos, etc.
    Ok(())
}
