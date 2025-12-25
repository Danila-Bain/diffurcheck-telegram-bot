use base64::{Engine, prelude::BASE64_STANDARD};
use std::{io::Write, process::Command};
use chrono::{NaiveTime, TimeDelta, Utc};
use sqlx::PgPool;
use teloxide::{
    Bot, dispatching::UpdateFilterExt, dptree::{self, Handler}, macros::BotCommands, net::Download,
    payloads::SendMessageSetters, prelude::Requester, sugar::request::RequestReplyExt,
    types::{CallbackQuery, ChatId, InputFile, InputMedia, InputMediaPhoto, MediaDocument, Message,
    MessageCommon, MessageId, MessageKind, Update}
};
use uuid::Uuid;
use variant_generation::{VariantGeneratorInput, VariantGeneratorOutput};

use crate::{
    bot::{BotState, HandlerResult, MyDialogue, MyError, idle::{self, IdleState}},
};

#[derive(Debug, Clone)]
pub enum AssignmentState {
    AwaitingSolutions { submission_id: Uuid, variant_id: Uuid },
    AwaitingFinish { submission_id: Uuid, variant_id: Uuid },
}


#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "Доступные команды:")]
pub enum AssignmentCommand {
    #[command(description = "показать это сообщение.")]
    Help,
    #[command(description = "завершить выполнение задания.")]
    Finish,
    #[command(description = "выйти из этого задания без завершения.")]
    GoBack,
    #[command(description = "посмотреть текущее задание.")]
    GetProblems,
    #[command(description = "посмотреть правила отправки решений.")]
    GetRules,
    #[command(description = "посмотреть оправленные решения.")]
    GetSolutions,
    #[command(description = "сколько осталось времени?")]
    ShowTimeLeft,
}

pub fn assignment_handler()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![BotState::Assignment(assignment_state)]
        .branch(
            Update::filter_message().branch(
                dptree::case![AssignmentState::AwaitingSolutions { submission_id, variant_id }]
                .branch(teloxide::filter_command::<AssignmentCommand, HandlerResult>()
                    .branch(dptree::case![AssignmentCommand::Help].endpoint(commands::help))
                    .branch(dptree::case![AssignmentCommand::Finish].endpoint(commands::finish))
                    .branch(dptree::case![AssignmentCommand::GoBack].endpoint(commands::go_back))
                    .branch(dptree::case![AssignmentCommand::GetProblems].endpoint(commands::get_problems))
                    .branch(dptree::case![AssignmentCommand::GetRules].endpoint(commands::get_rules))
                    .branch(dptree::case![AssignmentCommand::GetSolutions].endpoint(commands::update_and_show_solutions))
                    .branch(dptree::case![AssignmentCommand::ShowTimeLeft].endpoint(commands::show_time_left))
                    )
                .branch(dptree::endpoint(awaiting_solutions)),
            ),
        )
        .branch(
            Update::filter_edited_message().branch(
                dptree::case![AssignmentState::AwaitingSolutions { submission_id, variant_id }].branch(dptree::endpoint(awaiting_solutions))
            )
        )
        .branch(
            Update::filter_callback_query().branch(
                dptree::case![AssignmentState::AwaitingFinish { submission_id, variant_id }]
                    .endpoint(awaiting_finish),
            ),
        )
}

pub mod commands {
    use crate::bot::idle::{self, IdleState};

    use super::*;
    use sqlx::query_scalar;
    use teloxide::{types::{InlineKeyboardButton, LinkPreviewOptions}, utils::command::BotCommands};


    pub async fn help(
        bot: Bot,
        dialogue: MyDialogue,
    ) -> HandlerResult {
        bot.send_message(dialogue.chat_id(), AssignmentCommand::descriptions().to_string())
            .await?;
        Ok(())
    }


    pub async fn finish(
        bot: Bot,
        dialogue: MyDialogue,
        (submission_id, variant_id): (Uuid, Uuid),
        pool: PgPool,
    ) -> HandlerResult {


        commands::update_and_show_solutions(bot.clone(), dialogue.clone(), (submission_id, variant_id), pool.clone()).await?;

        let keyboard = teloxide::types::InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Завершить", "finish"),
            InlineKeyboardButton::callback("Нет, продолжить", "continue"),
        ]]);
        bot.send_message(dialogue.chat_id(), "Завершить выполнение задания?")
            .reply_markup(keyboard)
            .await?;


        dialogue
            .update(BotState::Assignment(AssignmentState::AwaitingFinish {
                submission_id,
                variant_id,
            }))
        .await?;

        Ok(())
    }


    pub async fn go_back(
        bot: Bot,
        dialogue: MyDialogue,
        (submission_id, _variant_id): (Uuid, Uuid),
        pool: PgPool,
    ) -> HandlerResult {

        let student_id = query_scalar!(
           r#"select student_id from submission where id = $1"#, submission_id 
        ).fetch_one(&pool).await?;

        bot.send_message(dialogue.chat_id(), "Чтобы продолжить выполнение задания, просто заново его откройте.")
            .await?;

        dialogue.update(BotState::Idle(IdleState::AwaitingCommand {student_id})).await?;
        idle::help(bot, dialogue, student_id, pool).await?;

        Ok(())
    }


    pub async fn get_problems(
        bot: Bot,
        dialogue: MyDialogue,
        (_submission_id, variant_id): (Uuid, Uuid),
        pool: PgPool,
    ) -> HandlerResult {


        send_problems(bot, dialogue.chat_id(), variant_id, pool).await?;

        Ok(())
    }


    pub async fn get_rules(
        bot: Bot,
        dialogue: MyDialogue,
        (submission_id, _variant_id): (Uuid, Uuid),
        pool: PgPool,
    ) -> HandlerResult {

        let rec = sqlx::query!(
            r#"

            SELECT a.title, a.description, a.duration, ga.deadline
            FROM submission s
            INNER JOIN group_assignment ga ON s.group_assignment_id = ga.id
            INNER JOIN assignment a ON ga.assignment_id = a.id
            WHERE s.id = $1
            "#,
            submission_id
        )
            .fetch_one(&pool)
            .await? ;

        let (title, description, duration, deadline) = (rec.title, rec.description, rec.duration, rec.deadline);
        let deadline = match deadline {
            Some(dt) => dt.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string(),
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


        bot.send_message(dialogue.chat_id(), text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .link_preview_options(LinkPreviewOptions {
                is_disabled: true,
                url: None,
                prefer_small_media: false,
                prefer_large_media: false,
                show_above_text: false,
            })
        .await?;

        Ok(())
    }


    pub async fn update_and_show_solutions(
        bot: Bot,
        dialogue: MyDialogue,
        (submission_id, _variant_id): (Uuid, Uuid),
        pool: PgPool,
    ) -> HandlerResult {

        let msg_ids = sqlx::query_scalar!(
            r#"
            select message_id from submission_item where submission_id = $1
            "#,
            submission_id
        ).fetch_all(&pool).await?;

        let mut no_solutions = true;
        for msg_id in msg_ids.into_iter() {
            match bot.send_message(dialogue.chat_id(), "👀.").reply_to(MessageId(msg_id)).await {
                Err(_) => {
                    // message with msg_id has been deleted, so we delete the data
                    sqlx::query!(
                        r#"delete from submission_item where message_id = $1"# ,
                        msg_id
                    ).execute(&pool).await?;
                },
                Ok(_) => {no_solutions = false;},
            }
        }

        if no_solutions {
            bot.send_message(dialogue.chat_id(), "Пока не прислано ни одного файла с решениями.").await?;
        }

        Ok(())
    }


    pub async fn show_time_left(
        bot: Bot,
        dialogue: MyDialogue,
        (submission_id, _variant_id): (Uuid, Uuid),
        pool: PgPool) -> HandlerResult {

        let time_left = time_left(submission_id, pool.clone()).await?;

        let time_left_text = match time_left {
            None => "(оставшееся время не известно)".to_string(),
            Some(time_left) => {
                if time_left > TimeDelta::zero() {
                    format!("{} мин. до конца", time_left.num_minutes())
                } else {
                    format!("просрочено на {} мин.", time_left.num_minutes().abs())
                }
            },
        };

        bot.send_message(dialogue.chat_id(), format!("Оставшееся время: {time_left_text}")).await?;

        Ok(())
    }
}

pub async fn set_time_left_reminder(
    reminder_time: TimeDelta,
    bot: Bot,
    chat_id: ChatId,
    submission_id: Uuid,
    pool: PgPool
) -> HandlerResult {

    tokio::spawn(async move {
        tokio::time::sleep(reminder_time.to_std().unwrap_or(tokio::time::Duration::from_secs(0))).await;


        let is_finished = sqlx::query_scalar!(
           r#"select finished_at from submission where id = $1"#, submission_id
        ).fetch_optional(&pool).await?.unwrap().is_some();

        if !is_finished {
            let time_left = time_left(submission_id, pool.clone()).await?;
            let time_left_text = match time_left {
                None => "(оставшееся время не известно)".to_string(),
                Some(time_left) => {
                    if time_left > TimeDelta::zero() {
                        format!("{} мин. до конца", time_left.num_minutes())
                    } else {
                        format!("просрочено на {} мин.", time_left.num_minutes().abs())
                    }
                },
            };

            bot.send_message(chat_id, format!("Оставшееся время: {time_left_text}")).await?;
        }

        Result::<(), MyError>::Ok(())
    });

    Ok(())
}

pub async fn insert_new_submission_with_new_variant(
    student_id: Uuid,
    group_assignment_id: Uuid,
    pool: PgPool,
) -> Result<(Uuid, Uuid), MyError> {

    let assignment_id = sqlx::query_scalar!(
        r#"
        select assignment_id from group_assignment
        where id = $1
        "#,
        group_assignment_id
    ).fetch_one(&pool).await?;

    let generator = sqlx::query_scalar!(
        r#"
        select generator
        from assignment
        where id = $1
        "#,
        assignment_id
    )
        .fetch_one(&pool)
        .await?;

    let variant_no: i32 = 1 + sqlx::query_scalar!(
        r#"
        SELECT MAX(variant_no)
        FROM variant
        WHERE assignment_id = $1
        "#,
        assignment_id
    )
        .fetch_one(&pool)
        .await?
        .unwrap_or(0);

    let exe_path = std::env::current_exe()
        .unwrap()
        .with_file_name(generator.clone());

    let input = VariantGeneratorInput {
        variant_number: variant_no,
        generator,
    };

    let output = 
        &Command::new(exe_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(serde_json::to_string(&input)?.as_bytes())?;
                child.wait_with_output()
            })?
            .stdout;

    // log::debug!("Output from variant generator: {}", String::from_utf8_lossy(output));
    let output: VariantGeneratorOutput = serde_json::from_slice(
        output
    )?;

    fn decode(s: &String) -> Result<Vec<u8>, MyError> {
        Ok(BASE64_STANDARD.decode(s)?)
    }

    let problem_code = output.problem_code;
    let solution_code = output.solution_code;

    let problem_images = output
        .problem_images
        .iter()
        .map(decode)
        .collect::<Result<Vec<Vec<u8>>, _>>()?;
    let solution_images = output
        .solution_images
        .iter()
        .map(decode)
        .collect::<Result<Vec<Vec<u8>>, _>>()?;

    let variant_id = sqlx::query_scalar!(
        r#"
        insert into variant (variant_no, assignment_id, problem_code, solution_code, problem_images, solution_images)
        values ($1, $2, $3, $4, $5, $6)
        returning id
        "#,
        variant_no,
        assignment_id, 
        problem_code,
        solution_code,
        &problem_images,
        &solution_images,
    ).fetch_one(&pool)
        .await?;

    let submission_id = sqlx::query_scalar!(
        r#"
        insert into submission (student_id, variant_id, group_assignment_id)
        values ($1, $2, $3)
        returning id
        "#,
        student_id,
        variant_id,
        group_assignment_id,
    ).fetch_one(&pool).await?;

    Ok((submission_id, variant_id))
}

pub async fn send_problems(
    bot: Bot,
    chat_id: ChatId,
    variant_id: Uuid,
    pool: PgPool,
) -> HandlerResult {

    let (_problem_code, problem_images) = {
        let rec = sqlx::query!(
            r#" select problem_code, problem_images from variant where id = $1 "#, variant_id
        ).fetch_one(&pool).await?;
        (rec.problem_code, rec.problem_images)
    };

    let media_group = problem_images.into_iter().map(|image| InputMedia::Photo(InputMediaPhoto::new(InputFile::memory(image)))).collect::<Vec<_>>();

    bot.send_media_group(chat_id, media_group).await?;

    // bot.send_message(
    //     chat_id,
    //     format!("Исходный код задач на случай плохого интернета (typst):\
    //         <blockquote expandable><code>{problem_code}</code></blockquote>")
    // )
    //     .parse_mode(teloxide::types::ParseMode::Html)
    //     .await?;
    //
    Ok(())
}

pub async fn time_left(
    submission_id: Uuid,
    pool: PgPool) -> Result<Option<TimeDelta>, MyError> {

    let now = Utc::now();

    let (started_at, deadline, duration) = {
        let rec = sqlx::query!(
            r#"
            select s.started_at, ga.deadline, a.duration
            from submission s
            inner join group_assignment ga on s.group_assignment_id = ga.id
            inner join assignment a on ga.assignment_id = a.id
            where s.id = $1
            "#,
            submission_id
        ).fetch_one(&pool).await?;
        (rec.started_at, rec.deadline, rec.duration)
    };
    let time_left = {
        let duration_left = duration.map(|d| (d - NaiveTime::from_hms_opt(0, 0, 0).unwrap()) - (now - started_at));
        let deadline_left = deadline.map(|d| d - now);

        match (duration_left, deadline_left) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    };

    Ok(time_left)
}

pub async fn awaiting_solutions(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (submission_id, variant_id): (Uuid, Uuid),
    pool: PgPool,
) -> HandlerResult {

    if sqlx::query_scalar!(
        r#"select finished_at from submission where id = $1"#, 
        submission_id
        ).fetch_one(&pool).await?.is_some() {

        bot.send_message(
            dialogue.chat_id(),
            "Время закончилось, выполнение задания уже завершено."
        ).await?;

        commands::update_and_show_solutions(bot.clone(), dialogue.clone(), (submission_id, variant_id), pool.clone()).await?;
            
        finish_assignment(bot, dialogue, submission_id, pool).await?;

        return Ok(());
    };

    let MessageKind::Common(MessageCommon{media_kind: data, ..}) = msg.kind else {
        bot.send_message(
            dialogue.chat_id(),
            "Данный тип сообщений не поддерживается (попробуйте обычные сообщения)."
        ).await?;
        return Ok(());
    };

    use teloxide::types::MediaKind::*;

    match data {
        Text(_) => {
            bot.send_message(
            dialogue.chat_id(),
                "Неизвестная команда, попробуйте /help."
            ).await?;
        },
        Photo(_) => {
            bot.delete_message(dialogue.chat_id(), msg.id).await?;
            bot.send_message(
            dialogue.chat_id(),
                "Пожалуйста, пришлите фотографии в виде несжатых файлов (см. /get_rules)"
            ).await?;
        },
        Document(MediaDocument {document, ..} ) => {

            let mime = document.mime_type.map(|m| m.to_string()).unwrap_or("bin".into());
            let file = bot.get_file(document.file.id).await?;
            let mut data: Vec<u8> = Vec::new();
            bot.download_file(&file.path, &mut data).await?;

            let pages = match mime.as_str() {
                "application/pdf" => {
                    let doc = lopdf::Document::load_mem(&data)?;
                    doc.get_pages().len()
                },
                _ => 1,
            } as i32;

            sqlx::query!(
                r#"
                insert into submission_item (submission_id, message_id, data, pages, extension)
                values ($1, $2, $3, $4, $5)
                on conflict(submission_id, message_id)
                do update set
                    data = excluded.data,
                    extension = excluded.extension,
                    pages = excluded.pages
                "#,
                submission_id,
                msg.id.0,
                data,
                pages,
                mime
            ).execute(&pool).await?;

            bot.send_message(
                dialogue.chat_id(), 
                "✍️."
            ).reply_to(msg.id)
            .await?;
        }

        _ => {
            bot.send_message(
                dialogue.chat_id(), 
                "Данный тип сообщений не поддерживается"
            ).await?;
        }
    };

    Ok(())
}


pub async fn awaiting_finish(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (submission_id, variant_id): (Uuid, Uuid),
    pool: PgPool,
) -> HandlerResult {
    let answer = q.data.clone().unwrap();
    let msg = q.regular_message().unwrap();

    if answer == "finish".to_string()  {
        bot.edit_message_text(dialogue.chat_id(), msg.id, "Выполнение задания завершено!").await?;
        finish_assignment(bot, dialogue, submission_id, pool).await?;
    } else {
        bot.delete_message(dialogue.chat_id(), msg.id).await?;

        dialogue.update(BotState::Assignment(AssignmentState::AwaitingSolutions {
            submission_id,
            variant_id,
        })).await?;
    }  

    Ok(())
}


pub async fn finish_assignment (
    bot: Bot,
    dialogue: MyDialogue,
    submission_id: Uuid,
    pool: PgPool,
) -> HandlerResult {
        // bot.edit_message_text(dialogue.chat_id(), msg.id, "Выполнение задания завершено!").await?;

        let student_id = sqlx::query_scalar!(
            r#"
            update submission
            set finished_at = $1
            where id = $2
            returning student_id
            "#,
            Utc::now(),
            submission_id,
        ).fetch_one(&pool).await?;


        dialogue.update(BotState::Idle(IdleState::AwaitingCommand {
            student_id
        })).await?;
        idle::help(bot, dialogue, student_id, pool).await?;

    Ok(())
}
