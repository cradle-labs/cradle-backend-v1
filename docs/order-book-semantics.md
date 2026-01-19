# Order Book Semantics: Bid/Ask Mapping

This document explains how the order book fields map to intuitive "amount in" and "amount out" concepts.

## Field Mappings

| Order Book Field | Intuitive Name | Description |
|------------------|----------------|-------------|
| `ask_asset` | `asset_in` | The asset you are **giving** (offering) |
| `ask_amount` | `amount_in` | How much you are **giving** |
| `bid_asset` | `asset_out` | The asset you **want** (bidding for) |
| `bid_amount` | `amount_out` | How much you **want** to receive |

## Key Insight

The naming comes from the perspective of a traditional order book:
- **Bid** = What you're bidding for (what you want)
- **Ask** = What you're asking in exchange (what you're offering)

When you place an order:
- Your `ask_asset` gets **locked** (you're giving this)
- You receive `bid_asset` when the order fills (you want this)

## Price Calculation

```
price = ask_amount / bid_amount
      = amount_in / amount_out
      = (what you give) / (what you want)
```

### Example

If you want to buy 100 TOKEN_B by offering 500 TOKEN_A:

| Field | Value | Meaning |
|-------|-------|---------|
| `ask_asset` | TOKEN_A | You're giving TOKEN_A |
| `ask_amount` | 500 | You're giving 500 TOKEN_A |
| `bid_asset` | TOKEN_B | You want TOKEN_B |
| `bid_amount` | 100 | You want 100 TOKEN_B |
| `price` | 5.0 | 500/100 = 5 TOKEN_A per TOKEN_B |

## Order Matching Logic

Orders match when they are complementary:
```sql
-- From the matching query
ob.bid_asset = io.ask_asset   -- Maker wants what taker is giving
ob.ask_asset = io.bid_asset   -- Maker is giving what taker wants
```

### Matching Example

**Order A (Maker):**
- Giving: 500 TOKEN_A (`ask_asset`, `ask_amount`)
- Wanting: 100 TOKEN_B (`bid_asset`, `bid_amount`)
- Price: 5.0

**Order B (Taker):**
- Giving: 100 TOKEN_B (`ask_asset`, `ask_amount`)
- Wanting: 500 TOKEN_A (`bid_asset`, `bid_amount`)
- Price: 0.2

These orders match because:
- Order A's `ask_asset` (TOKEN_A) = Order B's `bid_asset` (TOKEN_A)
- Order A's `bid_asset` (TOKEN_B) = Order B's `ask_asset` (TOKEN_B)

## Trade Execution

When a trade executes:

| Trade Field | Value | Description |
|-------------|-------|-------------|
| `maker_filled_amount` | Amount of maker's `ask_asset` transferred | What maker gave to taker |
| `taker_filled_amount` | Amount of taker's `ask_asset` transferred | What taker gave to maker |

The execution uses the **maker's price** (maker's ratio):
```rust
maker_ratio = maker.remaining_bid_amount / maker.remaining_ask_amount
```

## Asset Locking

When an order is placed:
- `ask_asset` (what you're giving) is **locked** in the system
- This prevents double-spending

When an order is cancelled:
- `ask_asset` is **unlocked** and returned to the user

```rust
// From operations.rs - cancellation unlocks ask_asset
unlock_asset(config, conn, order_data.wallet,
    order_data.ask_asset,   // The asset that was locked
    remaining_ask_amount    // The unfilled amount
)
```

## UI Form Mapping

In the admin UI, the form uses intuitive names:

```
User Input          →  Order Book Field
─────────────────────────────────────────
asset_in            →  ask_asset
amount_in           →  ask_amount
asset_out           →  bid_asset
amount_out          →  bid_amount
price (calculated)  →  amount_in / amount_out
```

## Summary

| Concept | You Give | You Receive |
|---------|----------|-------------|
| Asset | `ask_asset` / `asset_in` | `bid_asset` / `asset_out` |
| Amount | `ask_amount` / `amount_in` | `bid_amount` / `amount_out` |
| Locked? | Yes (on order placement) | No (received on fill) |

**Remember:**
- `ask` = what you **ask** in exchange = what you **give**
- `bid` = what you **bid** for = what you **want**
