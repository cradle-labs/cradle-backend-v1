-- Your SQL goes here


create type transaction_type as enum (
    'lock',
    'unlock',
    'lend',
    'borrow',
    'repay',
    'liquidate',
    'fill_order',
    'withdraw',
    'transfer',
    'buy_listed',
    'sell_listed',
    'listing_beneficiary_withdrawal'
);


create table AccountAssetsLedger (
    id uuid primary key default uuid_generate_v4(),
    timestamp timestamp not null default now(),
    transaction text,
    from_address text not null,
    to_address text not null,
    asset uuid not null references asset_book(id),
    transaction_type transaction_type not null,
    amount numeric not null,
    refference text -- ref can be any link to an existing record e.g for unlock an order trade, for lend a pool transaction etc
)
