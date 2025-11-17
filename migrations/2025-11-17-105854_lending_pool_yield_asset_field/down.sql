-- This file should undo anything in `up.sql`

alter table LendingPool drop column if exists yield_asset;
