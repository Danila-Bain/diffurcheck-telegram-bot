use sqlx::{PgPool, query_scalar};
use teloxide::{
    Bot,
    dispatching::{HandlerExt, UpdateFilterExt, dialogue::InMemStorage},
    dptree::{self, Handler},
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{CallbackQuery, Message, Update},
    utils::command::BotCommands,
};
use uuid::Uuid;

use crate::bot::{BotState, HandlerResult, MyDialogue};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "Доступные команды:")]
pub enum IdleCommand {
    #[command(description = "показать это сообщение.")]
    Help,
    #[command(description = "показать доступные задания.")]
    ShowAssignments,
}

#[derive(Debug, Clone)]
pub enum IdleState {
    AwaitingCommand {
        student_id: Uuid,
    },
    ViewAssignment {
        student_id: Uuid,
        assignment_id: Uuid,
    },
}

pub fn idle_handler() -> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription>
{
    dptree::case![BotState::Idle(idle_state)]
        .branch(
            Update::filter_message().branch(
                dptree::case![IdleState::AwaitingCommand { student_id }]
                    .branch(
                        teloxide::filter_command::<IdleCommand, HandlerResult>()
                            .branch(dptree::case![IdleCommand::Help].endpoint(help))
                            .branch(
                                dptree::case![IdleCommand::ShowAssignments]
                                    .endpoint(show_assignments),
                            ),
                    )
                    .branch(dptree::endpoint(unknown_command)),
            ),
        )
        .branch(Update::filter_callback_query().branch(
            dptree::case![IdleState::AwaitingCommand { student_id }].endpoint(idle_callback_query),
        ))
}

pub async fn help(
    bot: Bot,
    _dialogue: MyDialogue,
    msg: Message,
    _student_id: Uuid,
    _pool: PgPool,
) -> HandlerResult {
    bot.send_message(msg.chat.id, IdleCommand::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn unknown_command(
    bot: Bot,
    _dialogue: MyDialogue,
    msg: Message,
    _student_id: Uuid,
    _pool: PgPool,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "Ошибка: Неизвестная команда.".to_string())
        .await?;
    help(bot, _dialogue, msg, _student_id, _pool).await?;
    Ok(())
}
pub async fn show_assignments(
    bot: Bot,
    _dialogue: MyDialogue,
    msg: Message,
    student_id: Uuid,
    pool: PgPool,
) -> HandlerResult {
    let group_id = query_scalar!(
        r#"
                    select group_id
                    from student
                    where id = $1
                    "#,
        student_id
    )
    .fetch_optional(&pool)
    .await?
    .unwrap();

    let assignment_titles: Vec<String> = query_scalar!(
        r#"
                    SELECT a.title
                    FROM assignment a
                    INNER JOIN group_assignment ga ON a.id = ga.assignment_id
                    WHERE ga.group_id = $1
                    "#,
        group_id
    )
    .fetch_all(&pool)
    .await?;

    fn button(s: String) -> teloxide::types::InlineKeyboardButton {
        teloxide::types::InlineKeyboardButton::callback::<&str, &str>(&s, &s)
    }
    let keyboard = teloxide::types::InlineKeyboardMarkup::new(
        assignment_titles
            .into_iter()
            .map(|title| vec![button(title)])
            .collect::<Vec<_>>(),
    );

    bot.send_message(msg.chat.id, "Выбирите задание:")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn idle_callback_query(
    _bot: Bot,
    _dialogue: MyDialogue,
    q: CallbackQuery,
    _student_id: Uuid,
    _pool: PgPool,
) -> HandlerResult {
    log::debug!("Callback query: {q:?}");

    Ok(())
}
