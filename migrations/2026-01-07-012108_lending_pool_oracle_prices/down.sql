-- This file should undo anything in `up.sql`
-- down.sql
DROP INDEX IF EXISTS idx_lending_pool_asset;
drop table if exists lending_pool_oracle_prices;