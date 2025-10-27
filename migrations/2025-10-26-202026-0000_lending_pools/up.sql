-- Your SQL goes here

create table if not exists LendingPool (
    id uuid not null primary key default uuid_generate_v4(),
    pool_address text not null,
    pool_contract_id text not null,
    reserve_asset uuid not null references asset_book(id),
    loan_to_value numeric not null,
    base_rate numeric not null,
    slope1 numeric not null,
    slope2 numeric not null,
    liquidation_threshold numeric not null,
    liquidation_discount numeric not null,
    reserve_factor numeric not null,
    name text,
    title text,
    description text,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
);


create table if not exists LendingPoolSnapShots
(
    id                  uuid      not null primary key default uuid_generate_v4(),
    lending_pool_id     uuid      not null references LendingPool (id),
    total_supply        numeric   not null,
    total_borrow        numeric   not null,
    available_liquidity numeric   not null,
    utilization_rate    numeric   not null,
    supply_apy          numeric   not null,
    borrow_apy          numeric   not null,
    created_at          timestamp not null             default now()
)

