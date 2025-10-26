-- This file should undo anything in `up.sql`

drop index if exists idx_unique_order_per_wallet_assets_mode;
drop index if exists idx_unique_order_per_wallet_market_mode;
drop index if exists idx_unique_order_per_wallet_market_assets;
drop index if exists idx_unique_order_per_market_assets;
drop index if exists idx_unique_order_per_wallet_assets;
drop index if exists idx_unique_order_per_wallet_market;
drop index if exists idx_orderbook_cancelled_at;
drop index if exists idx_orderbook_filled_at;
drop index if exists idx_orderbook_created_at;
drop index if exists idx_orderbook_ask_asset;
drop index if exists idx_orderbook_bid_asset;
drop index if exists idx_orderbook_fill_mode;
drop index if exists idx_orderbook_status;
drop index if exists idx_orderbook_market_id;
drop index if exists idx_orderbook_wallet;

drop table if exists OrderBook;

drop type if exists order_status;
drop type if exists fill_mode;