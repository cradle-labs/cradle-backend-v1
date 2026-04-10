# Cradle Backend API Documentation

> Base URL: `http://{HOST}:{PORT}` (default port: `6969`)

## Authentication

All endpoints except `/health` require a Bearer token:

```
Authorization: Bearer {API_SECRET_KEY}
```

## Response Format

Every response follows this wrapper:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

On error:

```json
{
  "success": false,
  "data": null,
  "error": "Error message"
}
```

HTTP status codes: `200` OK, `400` Bad Request, `401` Unauthorized, `404` Not Found, `500` Internal Error.

---

## REST Endpoints

### Health

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check (no auth) |

Response: `{ "status": "ok", "timestamp": "2025-01-01T00:00:00Z" }`

---

### Accounts

| Method | Path | Description |
|--------|------|-------------|
| GET | `/accounts/:id` | Get account by UUID |
| GET | `/accounts/linked/:linked_id` | Get account by linked ID (string) |
| GET | `/accounts/:account_id/wallets` | Get wallet for account |
| GET | `/wallets/:id` | Get wallet by UUID |
| GET | `/wallets/account/:account_id` | Get wallet by account UUID |
| GET | `/balances/:wallet_contract_id` | Get all token balances for a wallet (Hedera call, cached 30s) |
| GET | `/balance/:wallet_id/:asset_id` | Get specific asset balance (Hedera call, cached 30s) |

#### Account Object

```json
{
  "id": "uuid",
  "linked_account_id": "string",
  "created_at": "2025-01-01T00:00:00",
  "account_type": "Retail" | "Institutional" | "System",
  "status": "Unverified" | "Verified" | "Suspended" | "Closed"
}
```

#### Wallet Object

```json
{
  "id": "uuid",
  "cradle_account_id": "uuid",
  "address": "0x...",
  "contract_id": "0.0.12345",
  "created_at": "2025-01-01T00:00:00",
  "status": "Active" | "Inactive" | "Suspended"
}
```

#### `GET /balances/:wallet_contract_id`

Returns all token balances from Hedera. The path param is the wallet's `contract_id` (e.g., `0.0.12345`).

```json
{
  "success": true,
  "data": [
    { "token": "HBAR", "balance": "1000000" },
    { "token": "0.0.67890", "balance": "5000000" }
  ]
}
```

#### `GET /balance/:wallet_id/:asset_id`

Returns net balance after deductions (locked amounts).

```json
{
  "success": true,
  "data": {
    "balance": 4500000,
    "before_deductions": 5000000,
    "deductions": 500000,
    "decimals": 6
  }
}
```

---

### Assets

| Method | Path | Description | Cache |
|--------|------|-------------|-------|
| GET | `/assets` | Get all assets | 1 hour |
| GET | `/assets/:id` | Get asset by UUID | 1 hour |
| GET | `/assets/token/:token` | Get asset by token address | - |
| GET | `/assets/manager/:manager` | Get asset by manager address | - |

#### Asset Object

```json
{
  "id": "uuid",
  "asset_manager": "0x...",
  "token": "0x...",
  "created_at": "2025-01-01T00:00:00",
  "asset_type": "Bridged" | "Native" | "yield_bearing" | "chain_native" | "stablecoin" | "Volatile",
  "name": "US Dollar Coin",
  "symbol": "USDC",
  "decimals": 6,
  "icon": "https://..." | null
}
```

---

### Markets

| Method | Path | Description | Cache |
|--------|------|-------------|-------|
| GET | `/markets` | Get all markets | 10 min |
| GET | `/markets/:id` | Get market by UUID | 10 min |

#### Query Params for `GET /markets`

| Param | Type | Description |
|-------|------|-------------|
| `market_type` | string? | Filter by type |
| `status` | string? | Filter by status |
| `regulation` | string? | Filter by regulation |

> Note: filters are accepted but not yet wired to the query.

#### Market Object

```json
{
  "id": "uuid",
  "name": "TSLA/USDC",
  "description": "Tesla stock spot market" | null,
  "icon": "https://..." | null,
  "asset_one": "uuid (base asset)",
  "asset_two": "uuid (quote asset)",
  "created_at": "2025-01-01T00:00:00",
  "market_type": "Spot" | "Derivative" | "Futures",
  "market_status": "Active" | "InActive" | "Suspended",
  "market_regulation": "Regulated" | "Unregulated"
}
```

---

### Orders

| Method | Path | Description |
|--------|------|-------------|
| GET | `/orders` | Get all orders |
| GET | `/orders/:id` | Get order by UUID |

#### Query Params for `GET /orders`

| Param | Type | Description |
|-------|------|-------------|
| `wallet` | uuid? | Filter by wallet |
| `market_id` | uuid? | Filter by market |
| `status` | string? | `open`, `closed`, `cancelled` |
| `order_type` | string? | `limit`, `market` |
| `mode` | string? | `fill-or-kill`, `immediate-or-cancel`, `good-till-cancel` |

> Note: filters are accepted but not yet wired to the query.

#### Order Object

```json
{
  "id": "uuid",
  "wallet": "uuid",
  "market_id": "uuid",
  "bid_asset": "uuid (asset the placer wants to receive)",
  "ask_asset": "uuid (asset the placer is offering)",
  "bid_amount": "1000000",
  "ask_amount": "500000",
  "price": "2.0",
  "filled_bid_amount": "0",
  "filled_ask_amount": "0",
  "mode": "fill-or-kill" | "immediate-or-cancel" | "good-till-cancel",
  "status": "open" | "closed" | "cancelled",
  "order_type": "limit" | "market",
  "created_at": "2025-01-01T00:00:00",
  "filled_at": "2025-01-01T00:00:00" | null,
  "cancelled_at": null,
  "expires_at": null
}
```

> All amounts are BigDecimal strings representing the smallest unit (e.g., with 6 decimals, `1000000` = 1.0 tokens).

---

### Time Series (OHLCV Candles)

| Method | Path | Description | Cache |
|--------|------|-------------|-------|
| GET | `/time-series/history` | Get candle data | 15s |

#### Query Params

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `market` | uuid | yes | Market UUID |
| `asset_id` | uuid | yes | Asset UUID to get candles for |
| `interval` | string | yes | Candle interval (see below) |
| `duration_secs` | string | yes | How far back in seconds (e.g., `86400` for 24h) |

#### Supported Intervals

`15secs`, `30secs`, `45secs`, `1min`, `5min`, `15min`, `30min`, `1hr`, `4hr`, `1day`, `1week`

#### Candle Object

```json
{
  "id": "uuid",
  "market_id": "uuid",
  "asset": "uuid",
  "open": "150.50",
  "high": "155.00",
  "low": "149.00",
  "close": "153.25",
  "volume": "1000000",
  "created_at": "2025-01-01T00:00:00",
  "start_time": "2025-01-01T00:00:00",
  "end_time": "2025-01-01T00:01:00",
  "interval": "1min",
  "data_provider_type": "order_book" | "exchange" | "aggregated",
  "data_provider": "orderbook_trades" | null
}
```

---

### Listings

| Method | Path | Description |
|--------|------|-------------|
| GET | `/listings` | Get all listings (with optional filters) |
| GET | `/listings/:listing_id` | Get listing by UUID |

#### Query Params for `GET /listings`

| Param | Type | Description |
|-------|------|-------------|
| `company` | uuid? | Filter by company |
| `listed_asset` | uuid? | Filter by listed asset |
| `purchase_asset` | uuid? | Filter by purchase asset |
| `status` | string? | `Pending`, `Open`, `Closed`, `Paused`, `Cancelled` |

#### Listing Object

```json
{
  "id": "uuid",
  "listing_contract_id": "0.0.12345",
  "name": "Tesla IPO",
  "description": "...",
  "documents": "ipfs://...",
  "company": "uuid",
  "status": "Open",
  "created_at": "2025-01-01T00:00:00",
  "opened_at": "2025-01-01T00:00:00" | null,
  "stopped_at": null,
  "listed_asset": "uuid",
  "purchase_with_asset": "uuid",
  "purchase_price": "150.00",
  "max_supply": "1000000",
  "treasury": "uuid",
  "shadow_asset": "uuid"
}
```

---

### Lending Pools

| Method | Path | Description | Cache |
|--------|------|-------------|-------|
| GET | `/pools` | Get all lending pools | - |
| GET | `/pools/:id` | Get pool by UUID | - |
| GET | `/loans/:wallet_id` | Get loans for wallet | - |
| GET | `/pool-stats/:pool_id` | Get pool stats (Hedera) | 30s |
| GET | `/loan-position/:loan_id` | Get borrow position (Hedera) | 15s |
| GET | `/pools/deposit/:pool_id/:wallet_id` | Get deposit position (Hedera) | 15s |
| GET | `/loans/repayments/:loan_id` | Get loan repayment history | - |
| GET | `/loan/:loan_id` | Get total repaid amount | - |
| GET | `/oracle/:pool_id/:asset_id` | Get oracle price | - |

#### Pool Object

```json
{
  "id": "uuid",
  "pool_address": "0x...",
  "pool_contract_id": "0.0.12345",
  "reserve_asset": "uuid",
  "loan_to_value": "0.75",
  "base_rate": "0.02",
  "slope1": "0.04",
  "slope2": "0.75",
  "liquidation_threshold": "0.80",
  "liquidation_discount": "0.05",
  "reserve_factor": "0.10",
  "name": "USDC Pool" | null,
  "title": "..." | null,
  "description": "..." | null,
  "created_at": "2025-01-01T00:00:00",
  "updated_at": "2025-01-01T00:00:00",
  "yield_asset": "uuid",
  "treasury_wallet": "uuid",
  "reserve_wallet": "uuid",
  "pool_account_id": "uuid"
}
```

#### Loan Object

```json
{
  "id": "uuid",
  "account_id": "uuid",
  "wallet_id": "uuid",
  "pool": "uuid",
  "borrow_index": "1.05",
  "principal_amount": "1000000",
  "created_at": "2025-01-01T00:00:00",
  "status": "Active" | "Repaid" | "Liquidated",
  "transaction": "0x..." | null,
  "collateral_asset": "uuid"
}
```

#### Oracle Price Object

```json
{
  "id": "uuid",
  "lending_pool_id": "uuid",
  "asset_id": "uuid",
  "price": "150.00",
  "created_at": "2025-01-01T00:00:00",
  "updated_at": "2025-01-01T00:00:00"
}
```

---

### Faucet

| Method | Path | Description |
|--------|------|-------------|
| POST | `/faucet` | Airdrop test tokens to account |

#### Request Body

```json
{
  "asset": "uuid (asset to airdrop)",
  "account": "uuid (account to receive)"
}
```

---

### OnRamp (Ramper)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/onramp-request` | Initiate fiat-to-crypto payment |
| POST | `/onramp-callback` | Webhook callback from payment provider |

#### `POST /onramp-request` Body

```json
{
  "token": "uuid",
  "amount": "100.00",
  "wallet_id": "uuid",
  "result_page": "https://app.cradle.com/result",
  "email": "user@example.com"
}
```

Response:

```json
{
  "success": true,
  "data": {
    "reference": "ref_123",
    "authorization_url": "https://payment.provider.com/pay/...",
    "access_code": "abc123"
  }
}
```

---

## Mutation Endpoint

### `POST /process`

This is the main write endpoint. It accepts a JSON body representing an action, routed to the appropriate processor.

The body is a **tagged enum** — a JSON object with a single key that identifies the processor, and a nested object identifying the action.

### Place Order

```json
{
  "OrderBook": {
    "PlaceOrder": {
      "wallet": "uuid",
      "market_id": "uuid",
      "bid_asset": "uuid",
      "ask_asset": "uuid",
      "bid_amount": "1000000",
      "ask_amount": "500000",
      "price": "2.0",
      "mode": "good-till-cancel",
      "order_type": "limit",
      "expires_at": null
    }
  }
}
```

Response:

```json
{
  "success": true,
  "data": {
    "OrderBook": {
      "PlaceOrder": {
        "id": "uuid",
        "status": "Filled" | "Partial" | "Cancelled",
        "bid_amount_filled": "1000000",
        "ask_amount_filled": "500000",
        "matched_trades": ["uuid", "uuid"]
      }
    }
  }
}
```

### Create Account

```json
{
  "Accounts": {
    "CreateAccount": {
      "linked_account_id": "external-user-123",
      "account_type": "Retail",
      "status": "Verified"
    }
  }
}
```

Response: `{ "Accounts": { "CreateAccount": { "id": "uuid", "wallet_id": "uuid" } } }`

### Create Asset

```json
{
  "AssetBook": {
    "CreateNewAsset": {
      "asset_type": "Native",
      "name": "My Token",
      "symbol": "MTK",
      "decimals": 6,
      "icon": "https://..."
    }
  }
}
```

### Create Market

```json
{
  "Markets": {
    "CreateMarket": {
      "name": "MTK/USDC",
      "description": "My Token spot market",
      "icon": null,
      "asset_one": "uuid (base)",
      "asset_two": "uuid (quote)",
      "market_type": "Spot",
      "market_status": "Active",
      "market_regulation": "Unregulated"
    }
  }
}
```

### Supply Liquidity to Pool

```json
{
  "Pool": {
    "SupplyLiquidity": {
      "wallet": "uuid",
      "pool": "uuid",
      "amount": 1000000
    }
  }
}
```

### Borrow from Pool

```json
{
  "Pool": {
    "BorrowAsset": {
      "wallet": "uuid",
      "pool": "uuid",
      "amount": 500000,
      "collateral": "uuid (collateral asset)"
    }
  }
}
```

### Repay Loan

```json
{
  "Pool": {
    "RepayBorrow": {
      "wallet": "uuid",
      "loan": "uuid",
      "amount": 250000
    }
  }
}
```

### Purchase Listing

```json
{
  "Listing": {
    "Purchase": {
      "wallet": "uuid",
      "amount": "100.00",
      "listing": "uuid"
    }
  }
}
```

### Withdraw Tokens

```json
{
  "Accounts": {
    "WithdrawTokens": {
      "withdrawal_type": "Crypto",
      "to": "0x...",
      "amount": "1000000",
      "token": "0x... (token address)",
      "from": "uuid (wallet_id)"
    }
  }
}
```

---

## Socket.IO (Realtime)

Connect to the same host/port as the REST API. Uses the default `/` namespace.

### Subscribing to Rooms

Send these events to join/leave rooms. Payload: `{ "market_id": "uuid" }`

| Client Event | Room Joined | Data You'll Receive |
|-------------|-------------|---------------------|
| `subscribe:orderbook` | `orderbook:{market_id}` | Order lifecycle events |
| `subscribe:trades` | `trades:{market_id}` | Trade execution events |
| `subscribe:timeseries` | `timeseries:{market_id}` | New OHLCV candles |

Unsubscribe with: `unsubscribe:orderbook`, `unsubscribe:trades`, `unsubscribe:timeseries`

### Server Events

#### Orderbook Room (`orderbook:{market_id}`)

**`order:placed`** — New order added to the book

```json
{
  "id": "uuid",
  "market_id": "uuid",
  "wallet": "uuid",
  "bid_asset": "uuid",
  "ask_asset": "uuid",
  "bid_amount": "1000000",
  "ask_amount": "500000",
  "price": "2.0",
  "status": "Open",
  "order_type": "Limit"
}
```

**`order:filled`** — Order fully filled (status = `"Closed"`)

Same shape as `order:placed`.

**`order:updated`** — Order partially filled (status = `"Open"`)

Same shape as `order:placed`.

**`order:cancelled`** — Order cancelled (status = `"Cancelled"`)

Same shape as `order:placed`.

#### Trades Room (`trades:{market_id}`)

**`trade:executed`** — Trades matched and settled

```json
{
  "order_id": "uuid",
  "market_id": "uuid",
  "trade_ids": ["uuid", "uuid"],
  "bid_amount_filled": "1000000",
  "ask_amount_filled": "500000",
  "status": "Filled"
}
```

#### Timeseries Room (`timeseries:{market_id}`)

**`price-change`** — New OHLCV candle recorded

```json
{
  "market_id": "uuid",
  "asset": "uuid",
  "open": "150.50",
  "high": "155.00",
  "low": "149.00",
  "close": "153.25",
  "volume": "1000000",
  "start_time": "2025-01-01T00:00:00",
  "end_time": "2025-01-01T00:01:00",
  "interval": "1min",
  "data_provider_type": "order_book",
  "data_provider": "orderbook_trades"
}
```

### Example (JavaScript)

```js
import { io } from "socket.io-client";

const socket = io("http://localhost:6969");

// Subscribe to a market
socket.emit("subscribe:orderbook", { market_id: "550e8400-..." });
socket.emit("subscribe:trades", { market_id: "550e8400-..." });
socket.emit("subscribe:timeseries", { market_id: "550e8400-..." });

// Listen for events
socket.on("order:placed", (data) => {
  console.log("New order:", data);
});

socket.on("trade:executed", (data) => {
  console.log("Trade:", data);
});

socket.on("price-change", (data) => {
  console.log("New candle:", data);
});

// Cleanup
socket.emit("unsubscribe:orderbook", { market_id: "550e8400-..." });
```

---

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | yes | - | PostgreSQL connection string |
| `API_SECRET_KEY` | no | `default-secret-key` | Bearer token for API auth |
| `REDIS_URL` | no | `redis://127.0.0.1:6379` | Redis cache (optional, runs without) |
| `PORT` | no | `6969` | Server port |
| `DISABLE_ONCHAIN_INTERACTIONS` | no | `false` | Skip Hedera calls (for testing) |
| `RUST_LOG` | no | `info` | Log level |

---

## Notes for Frontend Development

1. **All numeric amounts are BigDecimal strings** — parse them as strings, not floats. They represent the smallest unit of the asset (e.g., with 6 decimals, `"1000000"` = 1.0 tokens). Divide by `10^decimals` for display.

2. **UUIDs are lowercase hyphenated** — `"550e8400-e29b-41d4-a716-446655440000"`.

3. **Timestamps are ISO 8601 without timezone** — `"2025-01-01T00:00:00"` (NaiveDateTime, interpreted as UTC).

4. **The `/process` mutation endpoint uses Rust enum serialization** — the body is a single-key object where the key is the processor name and the value is another single-key object where the key is the action name.

5. **Cached endpoints** will return stale data within their TTL. After a mutation (e.g., placing an order), you may want to rely on Socket.IO events for immediate UI updates rather than polling the REST endpoint.

6. **Socket.IO connection** shares the same host:port as the REST API. No separate WebSocket server.

7. **Order matching semantics**: `bid_asset` is what you want, `ask_asset` is what you offer. For a buy order on market `TSLA/USDC`: `bid_asset` = TSLA, `ask_asset` = USDC.
