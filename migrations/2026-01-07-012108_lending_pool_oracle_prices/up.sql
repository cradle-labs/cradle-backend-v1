-- Your SQL goes here
create table if not exists lending_pool_oracle_prices (
    id uuid primary key default uuid_generate_v4(),
    lending_pool_id uuid not null references LendingPool(id),
    asset_id uuid not null references asset_book(id),
    price numeric not null,
    recorded_at timestamp not null default now()
);

-- up.sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_lending_pool_asset 
ON lending_pool_oracle_prices (lending_pool_id, asset_id);