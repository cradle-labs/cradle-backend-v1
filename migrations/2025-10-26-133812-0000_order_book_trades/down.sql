-- This file should undo anything in `up.sql`

drop index if exists idx_orderbooktrades_settled_at;
drop index if exists idx_orderbooktrades_created_at;
drop index if exists idx_orderbooktrades_settlement_status;
drop index if exists idx_orderbooktrades_taker_order;
drop index if exists idx_orderbooktrades_maker_order;

drop index if exists idx_unique_orderbook_trades;

drop table if exists OrderBookTrades;

drop type settlement_status;