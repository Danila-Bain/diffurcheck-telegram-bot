use std::sync::Arc;

use sqlx::PgPool;
use teloxide::{
    Bot,
    dispatching::dialogue::InMemStorage,
    prelude::{Dialogue, Requester},
    types::{ChatId, InputFile},
};
use uuid::Uuid;

use crate::bot::{BotState, HandlerResult, MyError};

// pub mod compile;

pub async fn close_overdue_submissions(
    bot: Bot,
    storage: Arc<InMemStorage<BotState>>,
    pool: PgPool,
    acceptable_overdue_minutes: f64,
) -> HandlerResult {
    let records = sqlx::query!(
        r#"
        UPDATE submission s
        SET finished_at = now()
        FROM
            group_assignment ga,
            assignment a,
            student st
        WHERE
            s.group_assignment_id = ga.id
            AND ga.assignment_id = a.id
            AND st.id = s.student_id
            AND (
                (now() > ga.deadline + ($1 * interval '1 minute'))
                OR
                (now() > s.started_at + a.duration::interval + ($1 * interval '1 minute'))
            )
            AND s.finished_at IS NULL
        RETURNING st.id as "student_id", st.chat_id, s.id as "submission_id", s.variant_id
        "#,
        acceptable_overdue_minutes
    )
    .fetch_all(&pool)
    .await?;

    for record in records.into_iter() {
        let dialogue = Dialogue::new(storage.clone(), ChatId(record.chat_id));

        bot.send_message(
            ChatId(record.chat_id),
            "Время закончилось, выполнение задания завершено.",
        )
        .await?;

        crate::bot::assignment::commands::update_and_show_solutions(
            bot.clone(),
            dialogue.clone(),
            (record.submission_id, record.variant_id),
            pool.clone(),
        )
        .await?;

        crate::bot::assignment::finish_assignment(
            bot.clone(),
            dialogue.clone(),
            record.submission_id,
            pool.clone(),
        )
        .await?;
    }

    Ok(())
}

pub async fn process_finished_assignments(
    bot: Bot,
    pool: PgPool,
    acceptable_overdue_minutes: f64,
) -> HandlerResult {
    let not_compiled_group_assignment_ids = sqlx::query_scalar!(
        r#"
            select id from group_assignment
            where 
                now() > deadline + ($1 * interval '1 minute')
                and
                solutions is null
        "#,
        acceptable_overdue_minutes
    )
    .fetch_all(&pool)
    .await?;

    let admin_chat_ids = sqlx::query_scalar!(r#"select chat_id from admin_chat"#)
        .fetch_all(&pool)
        .await?;

    for group_assignment_id in not_compiled_group_assignment_ids.into_iter() {
        let pdf = compile_group_assignment_pdf(group_assignment_id, pool.clone()).await?;

        sqlx::query!(
            r#"
            update group_assignment
            set solutions = $1
            where id = $2
            "#,
            pdf,
            group_assignment_id
        )
        .execute(&pool)
        .await?;

        for chat_id in admin_chat_ids.iter() {
            bot.send_document(
                ChatId(*chat_id),
                InputFile::memory(pdf.clone())
                    .file_name(format!("solutions_{group_assignment_id}.pdf")),
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn compile_group_assignment_pdf(
    group_assignment_id: Uuid,
    pool: PgPool,
) -> Result<Vec<u8>, MyError> {
    let (title, description, duration, available_at, deadline, group_name) = {
        let rec = sqlx::query!(
            r#"
            select a.title, a.description, a.duration, ga.available_at, ga.deadline, g.name as "group_name"
            from group_assignment ga
            inner join assignment a on ga.assignment_id = a.id
            inner join "group" g on ga.group_id = g.id
            where
                ga.id = $1
        "#,
            group_assignment_id
        )
        .fetch_one(&pool)
        .await?;
        (
            rec.title,
            rec.description,
            rec.duration,
            rec.available_at,
            rec.deadline,
            rec.group_name,
        )
    };

    let submission_ids = sqlx::query_scalar!(
        r#"
            select id from submission where group_assignment_id = $1
        "#,
        group_assignment_id
    )
    .fetch_all(&pool)
    .await?;

    let mut submissions = vec![];
    for submission_id in submission_ids.into_iter() {
        let rec = sqlx::query!(
            r#"
                select st.full_name as "student_name", 
                        v.variant_no, 
                        v.problem_images,
                        v.solution_images,
                        s.started_at, 
                        s.finished_at
                from submission s 
                    join variant v on s.variant_id = v.id
                    join student st on s.student_id = st.id
                where s.id = $1
                "#,
            submission_id
        )
        .fetch_one(&pool)
        .await?;

        let variant = Variant {
            number: rec.variant_no,
            problems: rec
                .problem_images
                .into_iter()
                .map(|img| Doc {
                    data: Bytes::new(img),
                    pages: 1,
                })
                .collect(),
            solutions: rec
                .solution_images
                .into_iter()
                .map(|img| Doc {
                    data: Bytes::new(img),
                    pages: 1,
                })
                .collect(),
        };

        let solutions = sqlx::query!(
            r#"
                select data, pages from submission_item where submission_id = $1
                "#,
            submission_id
        )
        .fetch_all(&pool)
        .await?;

        submissions.push(Submission {
            student_name: rec.student_name,
            variant,
            solutions: solutions
                .into_iter()
                .map(|sol| Doc {
                    data: Bytes::new(sol.data),
                    pages: sol.pages,
                })
                .collect(),
            started_at: rec
                .started_at
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            finished_at: rec.finished_at.map_or(
                "(не завершено)".to_string(),
                |finished_at| {
                    finished_at
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                },
            ),
        });
    }

    let template_text = include_str!("group_assignment_template.typ");

    let engine = typst_as_lib::TypstEngine::builder()
        .main_file(template_text)
        .search_fonts_with(
            TypstKitFontOptions::default()
                .include_system_fonts(false)
                // This line is not necessary, because thats the default.
                .include_embedded_fonts(true),
        )
        .build();

    let doc = engine
        .compile_with_input(GroupAssignmnet {
            title,
            description,
            group_name,
            available_at: available_at
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            duration: duration.map_or(
                "(без ограничения по времени)".to_string(),
                |duration| duration.format("%H ч. %M мин.").to_string(),
            ),
            deadline: deadline.map_or(
                "(без крайнего срока)".to_string(),
                |deadline| {
                    deadline
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                },
            ),
            submissions,
        })
        .output?;

    let options = Default::default();
    let pdf = typst_pdf::pdf(&doc, &options);

    match pdf {
        Ok(pdf) => return Ok(pdf),
        Err(_) => return Err("Could not produce pdf.".into()),
    }
    // std::fs::write("./output.pdf", pdf).expect("Could not write pdf.");
}

use derive_typst_intoval::{IntoDict, IntoValue};
use typst::foundations::{Bytes, Dict, IntoValue};
use typst_as_lib::typst_kit_options::TypstKitFontOptions;

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct GroupAssignmnet {
    title: String,
    description: String,
    group_name: String,
    available_at: String,
    duration: String,
    deadline: String,
    submissions: Vec<Submission>,
}
impl From<GroupAssignmnet> for Dict {
    fn from(value: GroupAssignmnet) -> Self {
        value.into_dict()
    }
}
#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Submission {
    student_name: String,
    variant: Variant,
    solutions: Vec<Doc>,
    started_at: String,
    finished_at: String,
}
#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Variant {
    number: i32,
    problems: Vec<Doc>,
    solutions: Vec<Doc>,
}
#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Doc {
    data: Bytes,
    pages: i32,
}
