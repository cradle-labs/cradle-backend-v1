-- This file should undo anything in `up.sql`
alter table LendingPool drop column reserve_wallet;
alter table LendingPool drop column treasury_wallet;
alter table LendingPool drop column pool_account_id;
