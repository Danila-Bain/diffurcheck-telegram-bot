// pub mod assignments;
pub mod bot;
pub mod db;
// pub mod submissions;
// mod util;
use crate::bot::BotState;
use sqlx::PgPool;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;
    let bot = Bot::from_env();

    let handler = bot::main_handler();
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<BotState>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

// pub async fn handle_update(
//     bot: Bot,
//     dialogue: Dialogue<BotState, InMemStorage<BotState>>,
//     msg: Message,
//     pool: PgPool,
// ) -> anyhow::Result<()> {
//     match dialogue.get().await? {
//         Some(BotState::NewUser) => {
//             bot.send_message(msg.chat.id, "Please enter your full name.")
//                 .await?;
//             dialogue.update(BotState::AwaitingFullName).await?;
//         }
//
//         Some(BotState::AwaitingFullName) => {
//             let name = msg.text().unwrap().to_string();
//             bot.send_message(msg.chat.id, "Choose your group.").await?;
//             dialogue
//                 .update(BotState::AwaitingGroup { full_name: name })
//                 .await?;
//         }
//
//         Some(BotState::AwaitingGroup { full_name }) => {
//             // save student
//             let student_id = crate::db::insert_student(
//                 &pool,
//                 &Student {
//                     id: new_uuid(),
//                     telegram_id: msg.from.clone().unwrap().id.0 as i64,
//                     full_name,
//                     group: msg.text().unwrap().parse()?,
//                     academic_year: crate::db::this_academic_year(),
//                     registered_at: now(),
//                 },
//             )
//             .await?;
//
//             dialogue.update(BotState::Idle { student_id }).await?;
//             bot.send_message(msg.chat.id, "Registration complete.")
//                 .await?;
//
//             let students = sqlx::query_as!(
//                     Student,
//                     r#"SELECT student_id as "id!", telegram_id, full_name, "group" as "group: _", academic_year, registered_at
//                     FROM student"#
//                 )
//                 .fetch_all(&pool)
//                 .await?;
//
//             bot.send_message(msg.chat.id, format!("{:?}", students))
//                 .await?;
//         }
//
//         Some(BotState::Idle { student_id: _ }) => {
//             // show available assignments
//         }
//
//         Some(BotState::InAssignment { session_id }) => {
//             // store incoming files
//             crate::submissions::storage::store_submission(session_id, &msg).await?;
//         }
//
//         Some(BotState::ViewingAssignment {
//             student_id: _,
//             assignment_id: _,
//         }) => todo!(),
//
//         Some(BotState::ConfirmCompletion { session_id: _ }) => todo!(),
//
//         None => {}
//     }
//
//     Ok(())
// }
