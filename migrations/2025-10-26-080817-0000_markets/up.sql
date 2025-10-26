-- Your SQL goes here

create type market_status as enum (
    'active',
    'inactive',
    'suspended'
);

create type market_type as enum (
    'spot',
    'derivative',
    'futures'
);

create type market_regulation as enum (
    'regulated',
    'unregulated'
);

create table if not exists markets (
    id uuid primary key default uuid_generate_v4(),
    name text not null,
    description text,
    icon text,
    asset_one uuid not null references asset_book(id),
    asset_two uuid not null references asset_book(id),
    created_at timestamp not null default now(),
    market_type market_type not null default 'spot',
    market_status market_status not null default 'active',
    market_regulation market_regulation not null default 'unregulated'
);

create index idx_markets_asset_one on markets(asset_one);
create index idx_markets_asset_two on markets(asset_two);
create index idx_markets_market_status on markets(market_status);
create index idx_markets_market_type on markets(market_type);
create index idx_markets_market_regulation on markets(market_regulation);
create unique index idx_unique_market_assets on markets(asset_one, asset_two);
