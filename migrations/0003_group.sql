create table if not exists "group" 
(
    id              uuid primary key                default uuid_generate_v1mc(),
    name            text                not null    collate "case_insensitive",
    academic_year   int                 not null    default current_academic_year(),
    created_at      timestamptz         not null    default now(),
    updated_at      timestamptz,
    unique (name, academic_year),
    check (academic_year between 2025 and 2100)
);
select trigger_updated_at('"group"');
