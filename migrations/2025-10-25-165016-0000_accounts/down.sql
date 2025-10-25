-- This file should undo anything in `up.sql`
drop index if exists idx_cradle_wallet_accounts_contract_id;
drop index if exists idx_cradle_wallet_accounts_address;
drop index if exists idx_cradle_wallet_accounts_cradle_account_id;
drop index if exists idx_cradle_accounts_linked_account_id;

drop table if exists CradleWalletAccounts;
drop table if exists CradleAccounts;

drop type if exists CradleAccountStatus;
drop type if exists CradleWalletStatus;
drop type if exists CradleAccountType;

drop extension if exists "uuid-ossp";