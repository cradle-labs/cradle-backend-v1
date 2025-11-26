-- Your SQL goes here

alter table LendingPool add column treasury_wallet uuid not null references CradleWalletAccounts(id);
alter table LendingPool add column reserve_wallet uuid not null references CradleWalletAccounts(id);
alter table LendingPool add column pool_account_id uuid not null references CradleAccounts(id);
