-- Your SQL goes here

create type settlement_status as enum (
    'matched',
    'settled',
    'failed'
);

create table if not exists OrderBookTrades (
    id uuid not null primary key default uuid_generate_v4(),
    maker_order_id uuid not null references OrderBook(id),
    taker_order_id uuid not null references OrderBook(id),
    maker_filled_amount numeric not null,
    taker_filled_amount numeric not null,
    settlement_tx text,
    settlement_status settlement_status not null default 'matched',
    created_at timestamp not null default now(),
    settled_at timestamp,
    constraint positive_filled_amounts check (maker_filled_amount > 0 and taker_filled_amount > 0)
);

create unique index idx_unique_orderbook_trades on OrderBookTrades(
                                                                  least(maker_order_id, taker_order_id),
                                                                    greatest(maker_order_id, taker_order_id)
    ) where settlement_status in ('matched');

create index idx_orderbooktrades_maker_order on OrderBookTrades(maker_order_id);
create index idx_orderbooktrades_taker_order on OrderBookTrades(taker_order_id);
create index idx_orderbooktrades_settlement_status on OrderBookTrades(settlement_status);
create index idx_orderbooktrades_created_at on OrderBookTrades(created_at);
create index idx_orderbooktrades_settled_at on OrderBookTrades(settled_at);
