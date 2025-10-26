-- This file should undo anything in `up.sql`

alter table OrderBook
    drop column order_type;

drop type order_type;