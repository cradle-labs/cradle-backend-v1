-- Your SQL goes here

create type asset_type as enum (
    'bridged',
    'native',
    'yield_bearing',
    'chain_native',
    'stablecoin',
    'volatile'
);

create table if not exists asset_book
(
    id uuid primary key default uuid_generate_v4(),
    asset_manager text not null unique,
    token text not null unique,
    created_at timestamp not null default now(),
    asset_type asset_type not null default 'native',
    name text not null,
    symbol text not null,
    decimals integer not null,
    icon text
);

