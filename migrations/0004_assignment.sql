create table if not exists assignment (
    id                  uuid primary key            default uuid_generate_v1mc(),
    title               text            not null    collate "case_insensitive",
    generator           text            not null,
    duration            time,
    created_at          timestamptz     not null    default now(),
    updated_at          timestamptz
);
select trigger_updated_at('"assignment"');
