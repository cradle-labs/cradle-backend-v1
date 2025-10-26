-- Your SQL goes here
create table if not exists AccountAssetBook (
    id uuid not null primary key default uuid_generate_v4(),
    asset_id uuid not null references asset_book(id),
    account_id uuid not null references CradleWalletAccounts(id),
    associated bool not null default false,
    kyced bool not null default false,
    associated_at timestamp,
    kyced_at timestamp,
    created_at timestamp not null default now(),
    constraint unique_asset_account unique (asset_id, account_id)
);

create index idx_account_asset_book_account_id on AccountAssetBook(account_id);