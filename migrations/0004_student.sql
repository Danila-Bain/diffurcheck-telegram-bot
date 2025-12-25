create table if not exists student (
    id              uuid primary key            default uuid_generate_v1mc(),
    group_id        uuid            not null    references "group"(id),
    telegram_id     bigint          not null,
    chat_id         bigint          not null,
    full_name       text            not null    collate "case_insensitive",
    created_at      timestamptz     not null    default now(),
    updated_at      timestamptz,
    unique (telegram_id, chat_id),
    unique (telegram_id, group_id)
);
select trigger_updated_at('"student"');
