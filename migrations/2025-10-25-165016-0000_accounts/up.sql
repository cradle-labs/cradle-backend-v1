-- Your SQL goes here
create extension if not exists "uuid-ossp";

create type CradleAccountType as enum ('retail', 'institutional');

create type CradleWalletStatus as enum ('active', 'inactive', 'suspended');

create type CradleAccountStatus as enum ('unverified', 'verified', 'suspended', 'closed');

create table if not exists CradleAccounts (
    id uuid primary key default uuid_generate_v4(),
    linked_account_id text not null unique,
    created_at timestamp not null default now(),
    account_type CradleAccountType not null default 'retail',
    status CradleAccountStatus not null default 'unverified'
);

create table if not exists CradleWalletAccounts (
    id uuid primary key default uuid_generate_v4(),
    cradle_account_id uuid not null references CradleAccounts(id),
    address text not null unique,
    contract_id text not null unique,
    created_at timestamp not null default now(),
    status CradleWalletStatus not null default 'inactive'
);

create index if not exists idx_cradle_accounts_linked_account_id on CradleAccounts(linked_account_id);
create index if not exists idx_cradle_wallet_accounts_cradle_account_id on CradleWalletAccounts(cradle_account_id);
create index if not exists idx_cradle_wallet_accounts_address on CradleWalletAccounts(address);
create index if not exists idx_cradle_wallet_accounts_contract_id on CradleWalletAccounts(contract_id);