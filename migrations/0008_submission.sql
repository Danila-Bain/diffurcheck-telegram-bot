create table if not exists submission (
    id                  uuid primary key            default uuid_generate_v1mc(),
    student_id          uuid            not null    references student(id)          on delete cascade,
    variant_id          uuid            not null    references variant(id)          on delete cascade,
    group_assignment_id uuid            not null    references group_assignment(id) on delete cascade,
    started_at          timestamptz     not null    default now(),
    finished_at         timestamptz,
    created_at          timestamptz     not null    default now(),
    updated_at          timestamptz,
    unique(student_id, group_assignment_id)
);
select trigger_updated_at('"submission"');

create table if not exists submission_item (
    id                  uuid primary key                default uuid_generate_v1mc(),
    submission_id       uuid                not null    references submission(id) on delete cascade,
    message_id          int                 not null,
    data                bytea               not null,
    pages               int                 not null,
    extension           text                not null,
    created_at          timestamptz         not null    default now(),
    updated_at          timestamptz,
    unique(submission_id, message_id)
);
select trigger_updated_at('"submission_item"');
