use sqlx::{PgPool, query_as};
use teloxide::{
    Bot,
    dispatching::{HandlerExt, UpdateFilterExt, dialogue::InMemStorage},
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
    db::{
        helpers::{active_groups, current_academic_year},
        model::{Group, Student},
    },
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
    fn group_button(group: Group) -> teloxide::types::InlineKeyboardButton {
        teloxide::types::InlineKeyboardButton::callback::<&str, &str>(&group.name, &group.name)
    }
    let groups = active_groups(&pool).await.unwrap_or(vec![]);
    let group_keyboard = teloxide::types::InlineKeyboardMarkup::new(
        groups
            .into_iter()
            .map(|group| vec![group_button(group)])
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

        let group_id = query_as!(
            Group,
            r#"
                select id, name, academic_year, created_at, updated_at from "group"
                where name = ($1) and academic_year = ($2);
            "#,
            group_name,
            current_academic_year(),
        )
        .fetch_optional(&pool)
        .await?
        .map(|group| group.id);

        if let Some(group_id) = group_id {
            let text = format!("Выбранная группа: {group_name}");

            if let Some(message) = q.regular_message() {
                bot.edit_message_text(message.chat.id, message.id, text)
                    .await?;
            }

            let chat_id = q.message.unwrap().chat().id;

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
    let full_name = msg.text().unwrap().to_string();
    let telegram_id = msg.clone().from.unwrap().id.0 as i64;

    let student = query_as!(
        Student,
        r#"
        insert into student (group_id, telegram_id, full_name)
        values ($1, $2, $3)
        returning
            id,
            group_id,
            telegram_id,
            full_name,
            created_at,
            updated_at
        "#,
        group_id,
        telegram_id,
        full_name,
    )
    .fetch_one(&pool)
    .await?;

    bot.send_message(msg.chat.id, "Регистрация пройдена!")
        .await?;

    idle::help(bot, dialogue.clone(), msg, student.id, pool).await?;
    // bot.send_message(msg.chat.id, idle::IdleCommand::descriptions().to_string())
    //     .await?;
    dialogue
        .update(BotState::Idle(IdleState::AwaitingCommand {
            student_id: student.id,
        }))
        .await?;

    // let students = sqlx::query_as!(
    //     Student,
    //     r#"select id, group_id, telegram_id, full_name, created_at, updated_at
    //     from student"#
    // )
    // .fetch_all(&pool)
    // .await?;
    // log::debug!("{students:?}");

    Ok(())
}
