create table if not exists variant (
    id              uuid primary key            default uuid_generate_v1mc(),
    variant_no      int not null,
    assignment_id   uuid not null references assignment(id) on delete cascade,
    created_at      timestamptz     not null    default now(),
    updated_at      timestamptz
);
select trigger_updated_at('"variant"');
