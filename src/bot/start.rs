use sqlx::PgPool;
use teloxide::{Bot, prelude::*, types::Message, utils::command::BotCommands};

use crate::{
    bot::{
        BotState, HandlerResult, MyDialogue,
        admin::AdminState,
        idle::{IdleCommand, IdleState},
        registration::{self, RegistrationState},
    },
};

pub fn start_handler()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![BotState::Start].branch(Update::filter_message().endpoint(start))
}

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message, pool: PgPool) -> HandlerResult {
    let Some(user) = msg.clone().from else {
        bot.send_message(msg.chat.id, "Сообщения из каналов не поддерживаются.")
            .await?;
        return Ok(());
    };
    let telegram_id = user.id.0 as i64;

    let is_admin = sqlx::query!(
        r#" select chat_id from admin_chat where chat_id = $1 "#,
        msg.chat.id.0
    )
    .fetch_optional(&pool)
    .await?
    .is_some();

    log::debug!("is admin? {}", is_admin);
    log::debug!("chat_id = {}", msg.chat.id.0);
//  5062133349
// -5062133349
    if is_admin {
        bot.send_message(msg.chat.id, "Запуск с привелегиями админа")
            .await?;

        dialogue
            .update(BotState::Admin(AdminState::AwaitingCommand))
            .await?;
        return Ok(());
    }

    let maybe_student = sqlx::query!(
        r#"
        select id, group_id, telegram_id, full_name, created_at, updated_at 
        from student
        where telegram_id = $1
        "#,
        telegram_id
    )
    .fetch_optional(&pool)
    .await?;

    match maybe_student {
        None => {
            bot.send_message(
                msg.chat.id,
                include_str!("long_messages/start_greeting.txt"),
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;

            registration::request_group(bot.clone(), msg.clone(), pool.clone()).await?;

            dialogue
                .update(BotState::Registration(RegistrationState::AwaitingGroup))
                .await?;
        }
        Some(student) => {
            bot.send_message(msg.chat.id, format!("{}, здравствуйте!", student.full_name))
                .await?;
            bot.send_message(msg.chat.id, IdleCommand::descriptions().to_string())
                .await?;
            dialogue
                .update(BotState::Idle(IdleState::AwaitingCommand {
                    student_id: student.id,
                }))
                .await?
        }
    }
    Ok(())
}
