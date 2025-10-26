-- This file should undo anything in `up.sql`

drop index if exists idx_markets_time_series_created_at;
drop index if exists idx_markets_time_series_end_time;
drop index if exists idx_markets_time_series_start_time;
drop index if exists idx_markets_time_series_asset;
drop index if exists idx_markets_time_series_market_id;

drop table if exists markets_time_series;

drop type if exists data_provider_type;

drop type if exists time_series_interval;