create table if not exists variant (
    id              uuid primary key            default uuid_generate_v1mc(),
    variant_no      int not null,
    assignment_id   uuid not null references assignment(id) on delete cascade,
    problem_code    text            not null,
    solution_code   text            not null,
    problem_images  bytea[]         not null,
    solution_images bytea[]         not null,
    created_at      timestamptz     not null    default now(),
    updated_at      timestamptz
);
select trigger_updated_at('"variant"');
