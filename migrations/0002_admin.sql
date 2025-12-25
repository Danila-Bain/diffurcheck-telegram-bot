create table admin_chat (
    chat_id bigint primary key,
    created_at      timestamptz         not null    default now(),
    updated_at      timestamptz
);
select trigger_updated_at('"admin_chat"');

insert into admin_chat(chat_id)
values (-5062133349);

