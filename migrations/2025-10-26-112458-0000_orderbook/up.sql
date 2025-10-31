-- Your SQL goes here

create type fill_mode as enum (
    'fill-or-kill',
    'immediate-or-cancel',
    'good-till-cancel'
);

create type order_status as enum (
    'open',
    'closed',
    'cancelled'
);

create table if not exists OrderBook (
    id uuid not null primary key default uuid_generate_v4(),
    wallet uuid not null references CradleWalletAccounts(id),
    market_id uuid not null references markets(id),
    bid_asset uuid not null references asset_book(id),
    ask_asset uuid not null references asset_book(id),
    bid_amount numeric not null,
    ask_amount numeric not null,
    price numeric not null,
    filled_bid_amount numeric not null default 0,
    filled_ask_amount numeric not null default 0,
    mode fill_mode not null default 'good-till-cancel',
    status order_status not null default 'open',
    created_at timestamp not null default now(),
    filled_at timestamp,
    cancelled_at timestamp,
    expires_at timestamp
    constraint different_assets check (bid_asset != ask_asset),
    constraint positive_amounts check (bid_amount > 0 and ask_amount > 0),
    constraint valid_filled_at check (
        (status = 'closed' and filled_at is not null) or
        (status != 'closed' and filled_at is null)
        )
                                     );

create index idx_orderbook_wallet on OrderBook(wallet);
create index idx_orderbook_market_id on OrderBook(market_id);
create index idx_orderbook_status on OrderBook(status);
create index idx_orderbook_fill_mode on OrderBook(mode);
create index idx_orderbook_bid_asset on OrderBook(bid_asset);
create index idx_orderbook_ask_asset on OrderBook(ask_asset);
create index idx_orderbook_created_at on OrderBook(created_at);
create index idx_orderbook_filled_at on OrderBook(filled_at);
create index idx_orderbook_cancelled_at on OrderBook(cancelled_at);