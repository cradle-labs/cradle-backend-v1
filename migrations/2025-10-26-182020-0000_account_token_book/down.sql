-- This file should undo anything in `up.sql`

drop index if exists idx_account_asset_book_account_id;

drop table if exists AccountAssetBook;