-- This file should undo anything in `up.sql`
alter table Loans drop column if exists collateral_asset;
