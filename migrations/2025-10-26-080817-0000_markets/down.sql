-- This file should undo anything in `up.sql`

drop index if exists idx_unique_market_assets;
drop index if exists idx_markets_market_regulation;
drop index if exists idx_markets_market_type;
drop index if exists idx_markets_market_status;
drop index if exists idx_markets_asset_two;
drop index if exists idx_markets_asset_one;

drop table if exists markets;

drop type if exists market_regulation;
drop type if exists market_type;
drop type if exists market_status;