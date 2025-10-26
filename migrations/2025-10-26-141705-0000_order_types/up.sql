-- Your SQL goes here


create type order_type as enum (
    'limit',
    'market'
);

alter table OrderBook
    add column order_type order_type not null default 'limit';