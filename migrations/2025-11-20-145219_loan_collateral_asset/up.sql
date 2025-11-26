-- Your SQL goes here

alter table Loans add column collateral_asset uuid not null references asset_book(id);
