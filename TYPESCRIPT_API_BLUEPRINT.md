# Cradle API → TypeScript Client Blueprint (detailed)

Base: Axum server in `src/main.rs`. Default host `0.0.0.0` and `PORT` env (falls back to `6969`).  
Auth: All routes except `GET /health` require `Authorization: Bearer <SECRET_KEY>` where `<SECRET_KEY>` is `API_SECRET_KEY` (`ApiConfig::secret_key`).  
Envelope: `ApiResponse<T> = { success: boolean; data?: T; error?: string }`.  
Numbers: Many Rust `BigDecimal` fields are serialized as JSON numbers/strings—treat as strings (`type Big = string`) to avoid precision loss.  
IDs/timestamps: `UUID = string`; timestamps are ISO strings (NaiveDateTime in DB).

```ts
type UUID = string;
type Big = string;

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}
```

## Core models (map Rust structs/enums to TS)
- `CradleAccountType`: `"retail" | "institutional" | "system"`
- `CradleAccountStatus`: `"unverified" | "verified" | "suspended" | "closed"`
- `CradleWalletStatus`: `"active" | "inactive" | "suspended"`

```ts
interface CradleAccountRecord {
  id: UUID;
  linked_account_id: string;
  created_at: string;
  account_type: CradleAccountType;
  status: CradleAccountStatus;
}

interface CradleWalletAccountRecord {
  id: UUID;
  cradle_account_id: UUID;
  address: string;
  contract_id: string;
  created_at: string;
  status: CradleWalletStatus;
}
```

- `AssetType`: `"bridged" | "native" | "yield_bearing" | "chain_native" | "stablecoin" | "volatile"`

```ts
interface AssetBookRecord {
  id: UUID;
  asset_manager: string;
  token: string; // solidity addr
  created_at: string;
  asset_type: AssetType;
  name: string;
  symbol: string;
  decimals: number;
  icon?: string | null;
}
```

- Markets: `MarketStatus = "active" | "inactive" | "suspended"`, `MarketType = "spot" | "derivative" | "futures"`, `MarketRegulation = "regulated" | "unregulated"`.

```ts
interface MarketRecord {
  id: UUID;
  name: string;
  description?: string | null;
  icon?: string | null;
  asset_one: UUID;
  asset_two: UUID;
  created_at: string;
  market_type: MarketType;
  market_status: MarketStatus;
  market_regulation: MarketRegulation;
}
```

- Order book enums: `FillMode = "fill-or-kill" | "immediate-or-cancel" | "good-till-cancel"`, `OrderStatus = "open" | "closed" | "cancelled"`, `OrderType = "limit" | "market"`.

```ts
interface OrderBookRecord {
  id: UUID;
  wallet: UUID;
  market_id: UUID;
  bid_asset: UUID;
  ask_asset: UUID;
  bid_amount: Big;
  ask_amount: Big;
  price: Big;
  filled_bid_amount: Big;
  filled_ask_amount: Big;
  mode: FillMode;
  status: OrderStatus;
  created_at: string;
  filled_at?: string | null;
  cancelled_at?: string | null;
  expires_at?: string | null;
  order_type: OrderType;
}

interface NewOrderBookRecord {
  wallet: UUID;
  market_id: UUID;
  bid_asset: UUID;
  ask_asset: UUID;
  bid_amount: Big;
  ask_amount: Big;
  price: Big;
  mode?: FillMode;
  expires_at?: string | null;
  order_type?: OrderType;
}
```

- Time series enums: `TimeSeriesInterval = "15secs" | "30secs" | "45secs" | "1min" | "5min" | "15min" | "30min" | "1hr" | "4hr" | "1day" | "1week"`. `DataProviderType = "order_book" | "exchange" | "aggregated"`.

```ts
interface MarketTimeSeriesRecord {
  id: UUID;
  market_id: UUID;
  asset: UUID;
  open: Big; high: Big; low: Big; close: Big; volume: Big;
  created_at: string;
  start_time: string;
  end_time: string;
  interval: TimeSeriesInterval;
  data_provider_type: DataProviderType;
  data_provider?: string | null;
}
```

- Lending: `LoanStatus = "active" | "repaid" | "liquidated"`. `PoolTransactionType = "supply" | "withdraw"`.

```ts
interface LendingPoolRecord {
  id: UUID;
  pool_address: string;
  pool_contract_id: string; // hedera 0.0.x form
  reserve_asset: UUID;
  yield_asset: UUID;
  treasury_wallet: UUID;
  reserve_wallet: UUID;
  pool_account_id: UUID;
  loan_to_value: Big;
  base_rate: Big;
  slope1: Big;
  slope2: Big;
  liquidation_threshold: Big;
  liquidation_discount: Big;
  reserve_factor: Big;
  name?: string | null;
  title?: string | null;
  description?: string | null;
  created_at: string;
  updated_at: string;
}

interface LendingPoolSnapShotRecord {
  id: UUID;
  lending_pool_id: UUID;
  total_supply: Big;
  total_borrow: Big;
  available_liquidity: Big;
  utilization_rate: Big;
  supply_apy: Big;
  borrow_apy: Big;
  created_at: string;
}

interface LoanRecord {
  id: UUID;
  account_id: UUID;
  wallet_id: UUID;
  pool: UUID;
  borrow_index: Big;
  principal_amount: Big;
  created_at: string;
  status: LoanStatus;
  transaction?: string | null;
  collateral_asset: UUID;
}

interface LoanRepaymentsRecord {
  id: UUID;
  loan_id: UUID;
  repayment_amount: Big;
  repayment_date: string;
  transaction?: string | null;
}
```

- Listings:

```ts
type ListingStatus = "pending" | "open" | "closed" | "paused" | "cancelled";

interface CompanyRow {
  id: UUID;
  name: string;
  description: string;
  listed_at?: string | null;
  legal_documents: string;
  beneficiary_wallet: UUID;
}

interface CradleNativeListingRow {
  id: UUID;
  listing_contract_id: string;
  name: string;
  description: string;
  documents: string;
  company: UUID;
  status: ListingStatus;
  created_at: string;
  opened_at?: string | null;
  stopped_at?: string | null;
  listed_asset: UUID;
  purchase_with_asset: UUID;
  purchase_price: Big;
  max_supply: Big;
  treasury: UUID;
  shadow_asset: UUID;
}
```

## HTTP endpoints (request/response shapes)
- `GET /health` → `ApiResponse<{ status: "ok" }>` (public, no auth).
- `POST /process` → `ApiResponse<ActionRouterOutput>`; body must be exactly one enum variant (see Action Router).

Accounts & wallets:
- `GET /accounts/:id` → `ApiResponse<CradleAccountRecord>`
- `GET /accounts/linked/:linked_id` → `ApiResponse<CradleAccountRecord>`
- `GET /accounts/:account_id/wallets` → `ApiResponse<CradleWalletAccountRecord>` (name plural; returns single wallet)
- `GET /wallets/:id` → `ApiResponse<CradleWalletAccountRecord>`
- `GET /wallets/account/:account_id` → `ApiResponse<CradleWalletAccountRecord>`
- `GET /balances/:account_id` → `ApiResponse<Array<{ token: string; balance: Big }>>` (HBAR + tokens from account balances)
- `GET /balance/:wallet_id/:asset_id` → `ApiResponse<{ balance: number; before_deductions: number; deductions: number; decimals: number }>`

Assets:
- `GET /assets` → `ApiResponse<AssetBookRecord[]>`
- `GET /assets/:id` → `ApiResponse<AssetBookRecord>`
- `GET /assets/token/:token` → `ApiResponse<AssetBookRecord>`
- `GET /assets/manager/:manager` → `ApiResponse<AssetBookRecord>`

Markets:
- `GET /markets` → `ApiResponse<MarketRecord[]>` (query params currently ignored)
- `GET /markets/:id` → `ApiResponse<MarketRecord>`

Orders:
- `GET /orders` → `ApiResponse<OrderBookRecord[]>` (filters accepted in query but ignored server-side)
- `GET /orders/:id` → `ApiResponse<OrderBookRecord>`

Time series:
- `GET /time-series/history?market=UUID&duration_secs=number|Big&interval=TimeSeriesInterval&asset_id=UUID`
  → `ApiResponse<MarketTimeSeriesRecord[]>`

Faucet:
- `POST /faucet` body `{ asset: UUID; account: UUID }` → `ApiResponse<void>` (associates + KYC + mints + airdrops)

Listings:
- `GET /listings` query `{ company?: UUID; listed_asset?: UUID; purchase_asset?: UUID; status?: ListingStatus }`
  → `ApiResponse<CradleNativeListingRow[]>`
- `GET /listings/:listing_id` → `ApiResponse<CradleNativeListingRow>`

Lending:
- `GET /pools` → `ApiResponse<LendingPoolRecord[]>`
- `GET /pools/:id` → `ApiResponse<LendingPoolRecord>`
- `GET /loans/:wallet` → `ApiResponse<LoanRecord[]>`
- `GET /pool-stats/:id` → `ApiResponse<GetPoolStatsOutput>`
- `GET /loan-position/:loan_id` → `ApiResponse<GetUserBorrowPositionOutput>`
- `GET /pools/deposit/:pool_id/:wallet_id` → `ApiResponse<GetUserDepositPositonOutput>`
- `GET /loans/repayments/:loan_id` → `ApiResponse<LoanRepaymentsRecord[]>`
- `GET /loan/:loan_id` → `ApiResponse<RepaymentAmount>`

## Action Router (`POST /process`)
Body is a single-key object; server rejects multiple top-level keys.

```ts
type ActionRouterInput =
  | { Accounts: AccountsProcessorInput }
  | { AssetBook: AssetBookProcessorInput }
  | { Markets: MarketProcessorInput }
  | { MarketTimeSeries: MarketTimeSeriesProcessorInput }
  | { OrderBook: OrderBookProcessorInput }
  | { Pool: LendingPoolFunctionsInput }
  | { Listing: CradleNativeListingFunctionsInput };

type ActionRouterOutput =
  | { Accounts: AccountsProcessorOutput }
  | { AssetBook: AssetBookProcessorOutput }
  | { Markets: MarketProcessorOutput }
  | { MarketTimeSeries: MarketTimeSeriesProcessorOutput }
  | { OrderBook: OrderBookProcessorOutput }
  | { Pool: LendingPoolFunctionsOutput }
  | { Listing: CradleNativeListingFunctionsOutput };
```

### Accounts
```ts
type AccountsProcessorInput =
  | { CreateAccount: { linked_account_id: string; account_type?: CradleAccountType; status?: CradleAccountStatus } }
  | { CreateAccountWallet: { cradle_account_id: UUID; status?: CradleWalletStatus } }
  | { UpdateAccountStatus: { cradle_account_id: UUID; status: CradleAccountStatus } }
  | { UpdateAccountType: { cradle_account_id: UUID; account_type: CradleAccountType } }
  | { UpdateAccountWalletStatusById: { wallet_id: UUID; status: CradleWalletStatus } }
  | { UpdateAccountWalletStatusByAccount: { cradle_account_id: UUID; status: CradleWalletStatus } }
  | { DeleteAccount: { ById: UUID } | { ByLinkedAccount: string } }
  | { DeleteWallet: { ById: UUID } | { ByOwner: UUID } }
  | { GetAccount: { ByID: UUID } | { ByLinkedAccount: string } }
  | { GetWallet: { ById: UUID } | { ByCradleAccount: UUID } }
  | { GetAccounts: {} }      // processor unimplemented
  | { GetWallets: {} }       // processor unimplemented
  | { AssociateTokenToWallet: { wallet_id: UUID; token: UUID } }
  | { GrantKYC: { wallet_id: UUID; token: UUID } }
  | { WithdrawTokens: { withdrawal_type: "Fiat" | "Crypto"; to: string; amount: Big; token: string; from: UUID } }
  | { HandleAssociateAssets: UUID }
  | { HandleKYCAssets: UUID };

type AccountsProcessorOutput =
  | { CreateAccount: { id: UUID; wallet_id: UUID } }
  | { CreateAccountWallet: { id: UUID } }
  | { UpdateAccountStatus: null }
  | { UpdateAccountType: null }
  | { UpdateAccountWalletStatus: null }          // some branches return UpdateAccountType; treat as void
  | { UpdateAccountWalletStatusById: null }
  | { UpdateAccountWalletStatusByAccount: null }
  | { GetAccount: CradleAccountRecord }
  | { GetWallet: CradleWalletAccountRecord }
  | { GetAccounts: null } // unimplemented
  | { GetWallets: null }  // unimplemented
  | { DeleteAccount: null }
  | { DeleteWallet: null }
  | { AssociateTokenToWallet: null }
  | { GrantKYC: null }
  | { WithdrawTokens: null }
  | { HandleAssociateAssets: null }
  | { HandleKYCAssets: null };
```

### Asset Book
```ts
type AssetBookProcessorInput =
  | { CreateNewAsset: { asset_type: AssetType; name: string; symbol: string; decimals: number; icon: string } }
  | { CreateExistingAsset: { asset_manager?: string; token: string; asset_type: AssetType; name: string; symbol: string; decimals: number; icon: string } }
  | { GetAsset: { ById: UUID } | { ByToken: string } | { ByAssetManager: string } };

type AssetBookProcessorOutput =
  | { CreateNewAsset: UUID }
  | { CreateExistingAsset: UUID }
  | { GetAsset: AssetBookRecord };
```

### Markets
```ts
type MarketProcessorInput =
  | { CreateMarket: { name: string; description?: string | null; icon?: string | null; asset_one: UUID; asset_two: UUID; market_type?: MarketType; market_status?: MarketStatus; market_regulation?: MarketRegulation } }
  | { UpdateMarketStatus: { market_id: UUID; status: MarketStatus } }
  | { UpdateMarketType: { market_id: UUID; market_type: MarketType } }
  | { UpdateMarketRegulation: { market_id: UUID; regulation: MarketRegulation } }
  | { GetMarket: UUID }
  | { GetMarkets: { status?: MarketStatus; market_type?: MarketType; regulation?: MarketRegulation } };

type MarketProcessorOutput =
  | { CreateMarket: UUID }
  | { UpdateMarketStatus: null }
  | { UpdateMarketType: null }
  | { UpdateMarketRegulation: null }
  | { GetMarket: MarketRecord }
  | { GetMarkets: MarketRecord[] };
```

### Market Time Series
```ts
type MarketTimeSeriesProcessorInput =
  | { AddRecord: { market_id: UUID; asset: UUID; open: Big; high: Big; low: Big; close: Big; volume: Big; start_time: string; end_time: string; interval?: TimeSeriesInterval; data_provider_type?: DataProviderType; data_provider?: string | null } }
  | { GetHistory: { market_id: UUID; duration_secs: Big; interval: TimeSeriesInterval; asset_id: UUID } };

type MarketTimeSeriesProcessorOutput =
  | { AddRecord: UUID }
  | { GetHistory: MarketTimeSeriesRecord[] };
```

### Order Book
```ts
type OrderBookProcessorInput =
  | { PlaceOrder: NewOrderBookRecord }
  | { GetOrder: UUID }
  | { GetOrders: { wallet?: UUID; market_id?: UUID; status?: OrderStatus; order_type?: OrderType; mode?: FillMode } };

type OrderFillStatus = "Partial" | "Filled" | "Cancelled";

type OrderBookProcessorOutput =
  | { PlaceOrder: { id: UUID; status: OrderFillStatus; bid_amount_filled: Big; ask_amount_filled: Big; matched_trades: UUID[] } }
  | { GetOrder: OrderBookRecord }
  | { GetOrders: OrderBookRecord[] };
```

### Lending Pools
```ts
type LendingPoolFunctionsInput =
  | { CreateLendingPool: { pool_address: string; pool_contract_id: string; reserve_asset: UUID; loan_to_value: Big; base_rate: Big; slope1: Big; slope2: Big; liquidation_threshold: Big; liquidation_discount: Big; reserve_factor: Big; name?: string | null; title?: string | null; description?: string | null; yield_asset: UUID; treasury_wallet: UUID; reserve_wallet: UUID; pool_account_id: UUID } }
  | { GetLendingPool: { ByName: string } | { ByAddress: string } | { ById: UUID } }
  | { CreateSnapShot: UUID }
  | { GetSnapShot: UUID }
  | { SupplyLiquidity: { wallet: UUID; pool: UUID; amount: number } } // amount: reserve tokens
  | { WithdrawLiquidity: { wallet: UUID; pool: UUID; amount: number } } // amount: yield tokens
  | { BorrowAsset: { wallet: UUID; pool: UUID; amount: number; collateral: UUID } }
  | { RepayBorrow: { wallet: UUID; loan: UUID; amount: number } }
  | { LiquidatePosition: { wallet: UUID; loan: UUID; amount: number } };

type LendingPoolFunctionsOutput =
  | { CreateLendingPool: UUID }
  | { GetLendingPool: LendingPoolRecord }
  | { CreateSnapShot: UUID }
  | { GetSnapShot: LendingPoolSnapShotRecord }
  | { SupplyLiquidity: UUID }   // pooltransactions row id
  | { WithdrawLiquidity: UUID } // pooltransactions row id
  | { BorrowAsset: UUID }       // loans row id
  | { RepayBorrow: null }
  | { LiquidatePosition: null };
```

### Listings (Cradle native)
```ts
type CradleNativeListingFunctionsInput =
  | { CreateCompany: { name: string; description: string; legal_documents: string } }
  | { CreateListing: { name: string; description: string; documents: string; company: UUID; asset: { Existing: UUID } | { New: { asset_type: AssetType; name: string; symbol: string; decimals: number; icon: string } }; purchase_asset: UUID; purchase_price: Big; max_supply: Big } }
  | { Purchase: { wallet: UUID; amount: Big; listing: UUID } }
  | { ReturnAsset: { wallet: UUID; amount: Big; listing: UUID } }
  | { WithdrawToBeneficiary: { amount: Big; listing: UUID } }
  | { GetStats: UUID }
  | { GetFee: { listing_id: UUID; amount: Big } };

type CradleNativeListingFunctionsOutput =
  | { CreateCompany: UUID }
  | { CreateListing: UUID }
  | { Purchase: null }
  | { ReturnAsset: null }
  | { WithdrawToBeneficiary: null }
  | { GetStats: ListingStats }      // from contract_integrator::cradle_native_listing
  | { GetFee: number };
```

## Usage example (`POST /process`)
```http
POST /process
Authorization: Bearer <SECRET_KEY>
Content-Type: application/json

{
  "OrderBook": {
    "PlaceOrder": {
      "wallet": "uuid",
      "market_id": "uuid",
      "bid_asset": "uuid",
      "ask_asset": "uuid",
      "bid_amount": "1000000",
      "ask_amount": "500000",
      "price": "2",
      "mode": "immediate-or-cancel",
      "expires_at": null,
      "order_type": "limit"
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
        "id": "…",
        "status": "Filled",
        "bid_amount_filled": "1000000",
        "ask_amount_filled": "500000",
        "matched_trades": ["…"]
      }
    }
  }
}
```

## Implementation notes / gotchas
- Top-level Action Router payload must have exactly one key; server rejects multiple.
- Big numeric fields should be treated as strings in TS to preserve precision.
- `GET /orders` currently ignores filters in handler; backend may need follow-up if filtering is required.
- `GetAccounts` and `GetWallets` actions are unimplemented server-side.
- Lending routes are now disambiguated: `GET /loan/:loan_id` (repay amount) vs `GET /loans/:wallet` (list loans). Borrow position route fixed to `/loan-position/:loan_id`.
- Listing creation input includes nested `asset` discriminated union (`Existing` | `New`) and derives multiple on-chain actions (mint/airdrop/association) under the hood—client only needs to send the shape above.
