use sqlx::PgPool;
use teloxide::{Bot, prelude::*, types::Message, utils::command::BotCommands};

use crate::{
    bot::{
        BotState, HandlerResult, MyDialogue,
        idle::{IdleCommand, IdleState},
        registration::{self, RegistrationState},
    },
    db::{
        helpers::active_groups,
        model::{Group, Student},
    },
};

pub fn start_handler()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![BotState::Start].branch(Update::filter_message().endpoint(start))
}

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message, pool: PgPool) -> HandlerResult {
    let user_id = msg.from.clone().unwrap().id.0 as i64;

    log::debug!("start function is called");

    let student = sqlx::query_as!(
        Student,
        r#"
        select id, group_id, telegram_id, full_name, created_at, updated_at 
        from student
        where telegram_id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await?;

    match student {
        None => {
            bot.send_message(
                msg.chat.id,
                r#"
Этот бот создан для дистанционного проведения контрольных работ по *Практике по дифференциальным уравнениям* (преподаватель: Баин Данила Денисович).

Что это и зачем:

- После регистрации вам предложат выбрать одно из доступных заданий. Задания можно начать выполнять в любое удобное время до указанного дедлайна, и на выполнение заданий будет выделено некоторое время.

- После начала выполнения задания присылаются задачи для выполнения, с указанием времени окончания. В указанное время нужно прислать решения в виде сканов решений либо в формате pdf, либо в виде картинок присланных файлами.

- После окончания дедлайна присланные решения компилируются в большой pdf файл, который также содержит варианты заданий и правильные ответы. Решения проверяются преподавателем с пометкой ошибок и тд и тп, и проверенный файл обратно пилится по студентам, и результаты проверки отправляются студенту.
"#
            )
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
