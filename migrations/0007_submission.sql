create table if not exists submission (
    id                  uuid primary key            default uuid_generate_v1mc(),
    student_id          uuid            not null references student(id)             on delete cascade,
    variant_id          uuid            not null references variant(id)             on delete cascade,
    group_assignment_id uuid            not null references group_assignment(id)    on delete cascade,
    data                bytea           not null,
    started_at          timestamptz     not null,
    finished_at         timestamptz     not null,
    completed           boolean         not null    default false,
    created_at          timestamptz     not null    default now(),
    updated_at          timestamptz
);
select trigger_updated_at('"submission"');
