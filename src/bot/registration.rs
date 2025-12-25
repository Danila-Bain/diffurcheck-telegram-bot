use sqlx::{PgPool};
use teloxide::{
    Bot,
    dispatching::{UpdateFilterExt, dialogue::GetChatId},
    dptree,
    prelude::*,
    types::{CallbackQuery, Message, Update},
};
use uuid::Uuid;

use crate::{
    bot::{
        BotState, HandlerResult, MyDialogue,
        idle::{self, IdleState},
    },
    db::helpers::current_academic_year,
};

#[derive(Debug, Clone, Default)]
pub enum RegistrationState {
    #[default]
    AwaitingGroup,
    AwaitingFullName {
        group_id: Uuid,
    },
}

pub fn registration_handler()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![BotState::Registration(reg_state)]
        .branch(
            Update::filter_callback_query()
                // .enter_dialogue::<CallbackQuery, InMemStorage<BotState>, BotState>()
                .branch(dptree::case![RegistrationState::AwaitingGroup].endpoint(awaiting_group)),
        )
        .branch(
            Update::filter_message()
                // .enter_dialogue::<Message, InMemStorage<BotState>, BotState>()
                .branch(
                    dptree::case![RegistrationState::AwaitingFullName { group_id }]
                        .endpoint(awaiting_full_name),
                ),
        )
}

pub async fn request_group(bot: Bot, msg: Message, pool: PgPool) -> HandlerResult {
    fn group_button(group_name: &str) -> teloxide::types::InlineKeyboardButton {
        teloxide::types::InlineKeyboardButton::callback::<&str, &str>(group_name, group_name)
    }

    let groups = sqlx::query_scalar!(
        r#"
            select name from "group"
            where academic_year = ($1)
        "#,
        current_academic_year()
    )
    .fetch_all(&pool)
    .await?;

    let group_keyboard = teloxide::types::InlineKeyboardMarkup::new(
        groups
            .into_iter()
            .map(|group| vec![group_button(group.as_str())])
            .collect::<Vec<_>>(),
    );
    bot.send_message(msg.chat.id, "Выбирите группу:")
        .reply_markup(group_keyboard)
        .await?;

    Ok(())
}

pub async fn awaiting_group(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    pool: PgPool,
) -> HandlerResult {
    log::debug!("Callback query: {q:?}");
    if let Some(ref group_name) = q.data {
        log::info!("Выбранная группа: {group_name}");

        let group_id = sqlx::query_scalar!(
            r#"
                select id from "group"
                where name = ($1) and academic_year = ($2);
            "#,
            group_name,
            current_academic_year(),
        )
        .fetch_optional(&pool)
        .await?;

        if let Some(group_id) = group_id {
            let Some(chat_id) = q.chat_id() else {
                dialogue.update(BotState::Start).await?;
                return Ok(());
            };
            let text = format!("Выбранная группа: {group_name}");

            bot.send_message(chat_id, text).await?;

            bot.send_message(chat_id, "Введите ваше ФИО:").await?;

            dialogue
                .update(BotState::Registration(
                    RegistrationState::AwaitingFullName { group_id },
                ))
                .await?;
        } else {
            let text = format!("Выбрана некорректная группа.");
            if let Some(message) = q.regular_message() {
                bot.edit_message_text(message.chat.id, message.id, text)
                    .await?;
            } else if let Some(id) = q.inline_message_id {
                bot.edit_message_text_inline(id, text).await?;
            }

            dialogue.update(BotState::Start).await?;
        }
    }
    Ok(())
}

async fn awaiting_full_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    group_id: Uuid,
    pool: PgPool,
) -> HandlerResult {
    let Some(full_name) = msg.text() else {
        bot.send_message(msg.chat.id, "Пожалуйста, пришлите своё ФИО:")
            .await?;
        return Ok(());
    };
    let Some(user) = msg.clone().from else {
        bot.send_message(msg.chat.id, "Сообщения из каналов не поддерживаются.")
            .await?;
        return Ok(());
    };
    let telegram_id = user.id.0 as i64;
    let chat_id = msg.chat.id.0 as i64;

    let student_id = sqlx::query_scalar!(
        r#"
        insert into student (group_id, telegram_id, chat_id, full_name)
        values ($1, $2, $3, $4)
        returning id
        "#,
        group_id,
        telegram_id,
        chat_id,
        full_name,
    )
    .fetch_one(&pool)
    .await?;

    bot.send_message(msg.chat.id, "Регистрация пройдена!")
        .await?;

    idle::help(bot, dialogue.clone(), student_id, pool).await?;
    dialogue
        .update(BotState::Idle(IdleState::AwaitingCommand {
            student_id: student_id,
        }))
        .await?;

    Ok(())
}
