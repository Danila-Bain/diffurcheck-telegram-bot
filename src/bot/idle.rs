use chrono::{DateTime, TimeDelta, Utc};
use sqlx::{PgPool, query, query_scalar};
use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    dptree::{self, Handler},
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::Requester,
    sugar::bot::BotMessagesExt,
    types::{CallbackQuery, InlineKeyboardButton, LinkPreviewOptions, Update},
    utils::command::BotCommands,
};
use uuid::Uuid;

use crate::bot::{
    BotState, HandlerResult, MyDialogue,
    assignment::{self, AssignmentState},
};

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
    AwaitingAssignmentStart {
        student_id: Uuid,
        group_assignment_id: Uuid,
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
                                    .endpoint(show_assignments_list),
                            ),
                    )
                    .branch(dptree::endpoint(unknown_command)),
            ),
        )
        .branch(
            Update::filter_callback_query()
                .branch(
                    dptree::case![IdleState::AwaitingCommand { student_id }]
                        .endpoint(show_assignment),
                )
                .branch(
                    dptree::case![IdleState::AwaitingAssignmentStart {
                        student_id,
                        group_assignment_id
                    }]
                    .endpoint(awaiting_assignment_start),
                ),
        )
}

pub async fn help(
    bot: Bot,
    dialogue: MyDialogue,
    _student_id: Uuid,
    _pool: PgPool,
) -> HandlerResult {
    bot.send_message(dialogue.chat_id(), IdleCommand::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn unknown_command(
    bot: Bot,
    dialogue: MyDialogue,
    _student_id: Uuid,
    _pool: PgPool,
) -> HandlerResult {
    bot.send_message(dialogue.chat_id(), "Ошибка: Неизвестная команда.".to_string())
        .await?;
    help(bot, dialogue, _student_id, _pool).await?;
    Ok(())
}

pub async fn show_assignments_list(
    bot: Bot,
    dialogue: MyDialogue,
    student_id: Uuid,
    pool: PgPool,
) -> HandlerResult {
    fn button_text(title: &String, deadline: &Option<DateTime<Utc>>) -> String {
        format!(
            "{}{}",
            title,
            if let Some(deadline) = deadline {
                format!(
                    " до {}",
                    deadline
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M")
                )
            } else {
                " (крайний срок сдачи не указан)".to_string()
            }
        )
    }
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

    let data = sqlx::query!(
        r#"
        SELECT a.title, a.generator, ga.deadline
        FROM assignment a
        INNER JOIN group_assignment ga ON a.id = ga.assignment_id
        WHERE ga.group_id = $1
        AND (ga.deadline IS NULL OR now() < ga.deadline)
        AND NOT ga.completed
        AND NOT EXISTS (
            SELECT 1
            FROM submission s
            WHERE s.student_id = $2
            AND s.group_assignment_id = ga.id
            AND s.finished_at IS NOT NULL
        )
        "#,
        group_id,
        student_id
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| (row.title, row.generator, row.deadline))
    .collect::<Vec<_>>();

    if data.is_empty() {
        bot.send_message(dialogue.chat_id(), "Сейчас нет доступных заданий.")
            .await?;
    } else {
        let keyboard = teloxide::types::InlineKeyboardMarkup::new(
            data.into_iter()
                .map(|(title, generator, deadline)| {
                    vec![InlineKeyboardButton::callback(
                        button_text(&title, &deadline),
                        generator,
                    )]
                })
                .collect::<Vec<_>>(),
        );

        bot.send_message(dialogue.chat_id(), "Выбирите задание:")
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

async fn show_assignment(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    student_id: Uuid,
    pool: PgPool,
) -> HandlerResult {
    let assignment_generator = q.data.clone().unwrap();

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

    let assignment_rec = query!(
        r#"
        SELECT ga.id, a.title, a.description, a.duration, ga.deadline
        FROM assignment a
        INNER JOIN group_assignment ga ON a.id = ga.assignment_id
        WHERE ga.group_id = $1
        AND a.generator = $2
        AND (ga.deadline IS NULL OR now() < ga.deadline)
        AND NOT ga.completed
        AND NOT EXISTS (
            SELECT 1
            FROM submission s
            WHERE s.student_id = $3
            AND s.group_assignment_id = ga.id
            AND s.finished_at IS NOT NULL
        )
        "#,
        group_id,
        assignment_generator,
        student_id
    )
    .fetch_optional(&pool)
    .await?
    .map(|rec| {
        (
            rec.id,
            rec.title,
            rec.description,
            rec.duration,
            rec.deadline,
        )
    });

    let message = q.regular_message().unwrap();

    match assignment_rec {
        Some((group_assignment_id, title, description, duration, deadline)) => {
            let deadline = match deadline {
                Some(dt) => dt
                    .with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
                None => "(не указано)".to_string(),
            };

            let time = match duration {
                None => "(до крайнего срока)".to_string(),
                Some(duration) => duration.format("%H часов %M минут").to_string(),
            };

            let text = format!(
                "{title}: \n\n{description}\n\nВремя на выполнение: {time}\n\nКрайний срок выполнения: {deadline}.\n\n{}",
                include_str!("long_messages/submission_rules.txt"),
            );

            bot.edit_message_text(message.chat.id, message.id, text)
                .parse_mode(teloxide::types::ParseMode::Html)
                .link_preview_options(LinkPreviewOptions {
                    is_disabled: true,
                    url: None,
                    prefer_small_media: false,
                    prefer_large_media: false,
                    show_above_text: false,
                })
                .await?;

            let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("Приступить", "start"),
                InlineKeyboardButton::callback("Назад", "back"),
            ]]);
            bot.send_message(message.chat.id, "Можно приступить прямо сейчас:")
                .reply_markup(keyboard)
                .await?;

            dialogue
                .update(BotState::Idle(IdleState::AwaitingAssignmentStart {
                    student_id,
                    group_assignment_id,
                }))
                .await?;
        }
        None => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                "Ошибка: выбранное задание недоступно.",
            )
            .await?;
        }
    }

    // dialogue
    //     .update(BotState::Idle(IdleState::AwaitingCommand {
    //         student_id: student.id,
    //     }))
    //     .await?;
    Ok(())
}

async fn awaiting_assignment_start(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (student_id, group_assignment_id): (Uuid, Uuid),
    pool: PgPool,
) -> HandlerResult {
    let answer = q.data.clone().unwrap();
    let msg = q.regular_message().unwrap();

    if answer == "start".to_string() {
        bot.edit_message_text(dialogue.chat_id(), msg.id, "Генерируем вариант...")
            .await?;

        let rec = query!(
            r#"
            select id, variant_id from submission
            where student_id = $1 and group_assignment_id = $2
            "#,
            student_id,
            group_assignment_id,
        )
        .fetch_optional(&pool)
        .await?
        .map(|rec| (rec.id, rec.variant_id));

        let (submission_id, variant_id) = if let Some(rec) = rec {
            rec
        } else {
            match assignment::insert_new_submission_with_new_variant(
                student_id,
                group_assignment_id,
                pool.clone(),
            )
            .await
            {
                Ok((submissinon_id, variant_id)) => (submissinon_id, variant_id),
                Err(_) => {
                    bot.edit_message_text(dialogue.chat_id(), msg.id, "Не удалось сгенерировать задачи. Для устранения ошибки, обратитесь к преподавателю.")
                        .await?;

                    dialogue
                        .update(BotState::Idle(IdleState::AwaitingCommand { student_id }))
                        .await?;
                    help(bot, dialogue, student_id, pool).await?;

                    return Ok(());
                }
            }
        };

        bot.edit_message_text(dialogue.chat_id(), msg.id, "Задачи: ")
            .await?;

        assignment::send_problems(bot.clone(), dialogue.chat_id(), variant_id, pool.clone()).await?;

        let time_left = assignment::time_left(submission_id, pool.clone()).await?;
        if let Some(time_left) = time_left {
            for time in [
                time_left + TimeDelta::minutes(15), // overdue
                time_left + TimeDelta::minutes(5),  // overdue
                time_left,
                time_left - TimeDelta::minutes(5),
                time_left - TimeDelta::minutes(15),
                time_left - TimeDelta::minutes(30),
                time_left - TimeDelta::minutes(60),
            ] {
                if time > TimeDelta::zero() {
                    assignment::set_time_left_reminder(
                        time,
                        bot.clone(),
                        dialogue.chat_id(),
                        submission_id,
                        pool.clone(),
                    )
                    .await?;
                }
            }
        }
        assignment::commands::help(bot, dialogue.clone()).await?;

        dialogue
            .update(BotState::Assignment(AssignmentState::AwaitingSolutions {
                submission_id,
                variant_id,
            }))
            .await?;
    } else {
        bot.delete(&msg).await?;

        dialogue
            .update(BotState::Idle(IdleState::AwaitingCommand { student_id }))
            .await?;
    }

    Ok(())
}
