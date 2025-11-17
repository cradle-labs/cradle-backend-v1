-- Your SQL goes here

alter table LendingPool add column if not exists yield_asset uuid not null references asset_book(id);
