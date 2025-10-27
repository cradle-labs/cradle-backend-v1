-- Your SQL goes here


create type pool_transaction_type as enum ('supply', 'withdraw');

create table if not exists PoolTransactions (
    id uuid not null primary key default uuid_generate_v4(),
    pool_id uuid not null references LendingPool(id),
    wallet_id uuid not null references CradleWalletAccounts(id),
    amount numeric not null,
    supply_index numeric not null,
    transaction_type pool_transaction_type not null default 'supply',
    yield_token_amount numeric not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    transaction text not null
)


