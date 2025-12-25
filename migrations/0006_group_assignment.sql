create table if not exists group_assignment (
    id                  uuid primary key            default uuid_generate_v1mc(),
    assignment_id       uuid            not null references assignment(id) on delete cascade,
    group_id            uuid            not null references "group"(id)    on delete cascade,
    solutions           bytea,
    graded_solutions    bytea,
    available_at        timestamptz     not null    default now(),
    deadline            timestamptz,
    completed           bool            not null    default false,
    created_at          timestamptz     not null    default now(),
    updated_at          timestamptz
);
select trigger_updated_at('"group_assignment"');


create or replace function insert_assignment_with_groups(
    p_title text,
    p_description text,
    p_generator text,
    p_duration time,
    p_groups jsonb
)
returns void
language plpgsql
as $$
declare
    missing_count int;
begin
    select count(*) into missing_count
    from jsonb_array_elements(p_groups) g
    left join "group" gr
      on gr.name = g->>'name'
     and gr.academic_year = current_academic_year()
    where gr.id is null;

    if missing_count > 0 then
        raise exception 'some groups do not exist';
    end if;

with new_assignment as (
    insert into assignment (title, description, generator, duration)
    values (p_title, p_description, p_generator, p_duration)
    returning id
),
groups_with_deadlines as (
    select
        (g->>'name')::text        as name,
        current_academic_year()  as academic_year,
        (g->>'deadline')::timestamptz as deadline
    from jsonb_array_elements(p_groups) as g
)
insert into group_assignment (group_id, assignment_id, deadline)
select
    gr.id,
    na.id,
    gwd.deadline
from new_assignment na
join groups_with_deadlines gwd
    on true
join "group" gr
    on gr.name = gwd.name
   and gr.academic_year = gwd.academic_year;

end;
$$;
