mod bot;
mod db;
mod submissions;

// mod util;
use crate::bot::BotState;
use sqlx::PgPool;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

const UPDATE_RATE_MINUTES: f64 = 5.;
const ACCEPTABLE_OVERDUE_MINUTES: f64 = 15.;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;
    let bot = Bot::from_env();
    let storage = InMemStorage::<BotState>::new();

    tokio::spawn({
        let pool = pool.clone();
        let bot = bot.clone();
        let storage = storage.clone();
        let admin_chat_id = ChatId(std::env::var("ADMIN_CHAT_ID")?.parse()?);

        async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs((60. * UPDATE_RATE_MINUTES) as u64)).await;
                match submissions::close_overdue_submissions(bot.clone(), storage.clone(), pool.clone(), ACCEPTABLE_OVERDUE_MINUTES).await {
                    Ok(_) => {},
                    Err(err) => log::error!("closue_overdue_sumbissions failed with {err}"),
                }
                match submissions::process_finished_assignments(bot.clone(), pool.clone(), admin_chat_id, ACCEPTABLE_OVERDUE_MINUTES).await {
                    Ok(_) => {},
                    Err(err) => log::error!("process_finished_assignments failed with {err}"),
                }
            }
        }
    });

    Dispatcher::builder(bot, bot::main_handler())
        .dependencies(dptree::deps![storage, pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
