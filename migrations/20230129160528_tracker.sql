-- Add migration script here
create table tracked_peers (
  id bigserial,
  info_hash bytea not null,
  endpoint text not null,
  inserted_at date not null default current_date,
  primary key (info_hash, endpoint)
);