-- Your SQL goes here

create type time_series_interval as enum (
    '1min',
    '5min',
    '15min',
    '30min',
    '1hr',
    '4hr',
    '1day',
    '1week'
);

create type data_provider_type as enum (
    'order_book',
    'exchange',
    'aggregated'
);

create table if not exists markets_time_series (
    id uuid primary key default uuid_generate_v4(),
    market_id uuid not null references markets(id),
    asset uuid not null references asset_book(id),
    open numeric not null,
    high numeric not null,
    low numeric not null,
    close numeric not null,
    volume numeric not null,
    created_at timestamp not null default now(),
    start_time timestamp not null,
    end_time timestamp not null,
    interval time_series_interval not null default '1min',
    data_provider_type data_provider_type not null default 'exchange',
    data_provider text
);

create index idx_markets_time_series_market_id on markets_time_series(market_id);
create index idx_markets_time_series_asset on markets_time_series(asset);
create index idx_markets_time_series_start_time on markets_time_series(start_time);
create index idx_markets_time_series_end_time on markets_time_series(end_time);
create index idx_markets_time_series_created_at on markets_time_series(created_at);
