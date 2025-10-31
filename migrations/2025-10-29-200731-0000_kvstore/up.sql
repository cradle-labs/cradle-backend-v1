-- Your SQL goes here

create table if not exists kvstore (
    key text not null unique primary key,
    value text
);