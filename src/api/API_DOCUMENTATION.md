# Cradle Back-End REST API Documentation

## Overview

This document provides comprehensive API documentation for the Cradle Back-End REST API built with Axum. The API provides RESTful endpoints for querying blockchain data and processing mutations through a unified action router system.

**Base URL:** `http://localhost:3000`

**Authentication:** All endpoints (except `/health`) require Bearer token authentication via `Authorization: Bearer <token>` header.

**Response Format:** All responses are wrapped in a standardized JSON structure with `success`, `data`, and `error` fields.

---

## Table of Contents

1. [Authentication](#authentication)
2. [Response Format](#response-format)
3. [Health Check](#health-check)
4. [Accounts API](#accounts-api)
5. [Assets API](#assets-api)
6. [Markets API](#markets-api)
7. [Orders API](#orders-api)
8. [Time Series API](#time-series-api)
9. [Lending Pools API](#lending-pools-api)
10. [Mutations API (POST /process)](#mutations-api-post-process)
11. [Enum Reference](#enum-reference)

---

## Authentication

### Bearer Token

All protected endpoints require an `Authorization` header with a Bearer token:

```
Authorization: Bearer <API_SECRET_KEY>
```

The `API_SECRET_KEY` is loaded from the environment variable `API_SECRET_KEY`.

### Example Request

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/accounts/550e8400-e29b-41d4-a716-446655440000
```

---

## Response Format

### Success Response

```json
{
  "success": true,
  "data": {
    // endpoint-specific data
  },
  "error": null
}
```

### Error Response

```json
{
  "success": false,
  "data": null,
  "error": "Description of the error"
}
```

### HTTP Status Codes

- `200 OK` - Successful request
- `400 Bad Request` - Invalid input parameters
- `401 Unauthorized` - Missing or invalid authentication token
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

---

## Health Check

### GET /health

Public endpoint for checking API status.

**Authentication:** Not required

**Response:**

```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

---

## Accounts API

### GET /accounts/{id}

Get a cradle account by UUID.

**Parameters:**
- `id` (path, required): Account UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/accounts/550e8400-e29b-41d4-a716-446655440000
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "linked_account_id": "external-account-123",
    "created_at": "2024-01-01T10:00:00",
    "account_type": "retail",
    "status": "verified"
  },
  "error": null
}
```

---

### GET /accounts/linked/{linked_id}

Get account by linked account identifier.

**Parameters:**
- `linked_id` (path, required): External linked account ID (string)

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/accounts/linked/external-account-123
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "linked_account_id": "external-account-123",
    "created_at": "2024-01-01T10:00:00",
    "account_type": "retail",
    "status": "verified"
  },
  "error": null
}
```

---

### GET /accounts/{account_id}/wallets

Get all wallets for an account.

**Parameters:**
- `account_id` (path, required): Account UUID

**Status:** Not yet implemented

---

### GET /wallets/{id}

Get a specific wallet by UUID.

**Parameters:**
- `id` (path, required): Wallet UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/wallets/660e8400-e29b-41d4-a716-446655440001
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "cradle_account_id": "550e8400-e29b-41d4-a716-446655440000",
    "address": "0.0.123456",
    "contract_id": "0x1a2b3c4d5e6f",
    "created_at": "2024-01-01T10:30:00",
    "status": "active"
  },
  "error": null
}
```

---

### GET /wallets/account/{account_id}

Get wallet by account ID.

**Parameters:**
- `account_id` (path, required): Account UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/wallets/account/550e8400-e29b-41d4-a716-446655440000
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "cradle_account_id": "550e8400-e29b-41d4-a716-446655440000",
    "address": "0.0.123456",
    "contract_id": "0x1a2b3c4d5e6f",
    "created_at": "2024-01-01T10:30:00",
    "status": "active"
  },
  "error": null
}
```

---

## Assets API

### GET /assets/{id}

Get an asset by UUID.

**Parameters:**
- `id` (path, required): Asset UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/assets/550e8400-e29b-41d4-a716-446655440000
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "asset_manager": "0x1234567890abcdef",
    "token": "0.0.12345",
    "created_at": "2024-01-01T08:00:00",
    "asset_type": "native",
    "name": "Hedera",
    "symbol": "HBAR",
    "decimals": 8,
    "icon": "https://example.com/hbar.png"
  },
  "error": null
}
```

---

### GET /assets/token/{token}

Get asset by token identifier.

**Parameters:**
- `token` (path, required): Token ID (e.g., "0.0.12345")

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/assets/token/0.0.12345
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "asset_manager": "0x1234567890abcdef",
    "token": "0.0.12345",
    "created_at": "2024-01-01T08:00:00",
    "asset_type": "native",
    "name": "Hedera",
    "symbol": "HBAR",
    "decimals": 8,
    "icon": "https://example.com/hbar.png"
  },
  "error": null
}
```

---

### GET /assets/manager/{manager}

Get asset by asset manager identifier.

**Parameters:**
- `manager` (path, required): Asset manager address

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/assets/manager/0x1234567890abcdef
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "asset_manager": "0x1234567890abcdef",
    "token": "0.0.12345",
    "created_at": "2024-01-01T08:00:00",
    "asset_type": "native",
    "name": "Hedera",
    "symbol": "HBAR",
    "decimals": 8,
    "icon": "https://example.com/hbar.png"
  },
  "error": null
}
```

---

## Markets API

### GET /markets/{id}

Get a market by UUID.

**Parameters:**
- `id` (path, required): Market UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/markets/750e8400-e29b-41d4-a716-446655440002
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "750e8400-e29b-41d4-a716-446655440002",
    "name": "HBAR/USDC",
    "description": "Hedera to USD Coin trading pair",
    "icon": "https://example.com/hbar-usdc.png",
    "asset_one": "550e8400-e29b-41d4-a716-446655440000",
    "asset_two": "650e8400-e29b-41d4-a716-446655440001",
    "created_at": "2024-01-01T09:00:00",
    "market_type": "spot",
    "market_status": "active",
    "market_regulation": "regulated"
  },
  "error": null
}
```

---

### GET /markets

Get all markets with optional filters.

**Query Parameters:**
- `market_type` (optional): "spot", "derivative", or "futures"
- `status` (optional): "active", "inactive", or "suspended"
- `regulation` (optional): "regulated" or "unregulated"

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  "http://localhost:3000/markets?market_type=spot&status=active"
```

**Example Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "750e8400-e29b-41d4-a716-446655440002",
      "name": "HBAR/USDC",
      "description": "Hedera to USD Coin trading pair",
      "icon": "https://example.com/hbar-usdc.png",
      "asset_one": "550e8400-e29b-41d4-a716-446655440000",
      "asset_two": "650e8400-e29b-41d4-a716-446655440001",
      "created_at": "2024-01-01T09:00:00",
      "market_type": "spot",
      "market_status": "active",
      "market_regulation": "regulated"
    }
  ],
  "error": null
}
```

---

## Orders API

### GET /orders/{id}

Get an order by UUID.

**Parameters:**
- `id` (path, required): Order UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/orders/850e8400-e29b-41d4-a716-446655440003
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "850e8400-e29b-41d4-a716-446655440003",
    "wallet": "550e8400-e29b-41d4-a716-446655440000",
    "market_id": "750e8400-e29b-41d4-a716-446655440002",
    "bid_asset": "550e8400-e29b-41d4-a716-446655440000",
    "ask_asset": "650e8400-e29b-41d4-a716-446655440001",
    "bid_amount": "1000.00",
    "ask_amount": "500.00",
    "price": "2.00",
    "filled_bid_amount": "500.00",
    "filled_ask_amount": "250.00",
    "mode": "good-till-cancel",
    "status": "open",
    "created_at": "2024-01-15T10:00:00",
    "filled_at": null,
    "cancelled_at": null,
    "expires_at": null,
    "order_type": "limit"
  },
  "error": null
}
```

---

### GET /orders

Get orders with optional filters.

**Query Parameters:**
- `wallet` (optional): Wallet UUID
- `market_id` (optional): Market UUID
- `status` (optional): "open", "closed", or "cancelled"
- `order_type` (optional): "limit" or "market"
- `mode` (optional): "fill-or-kill", "immediate-or-cancel", or "good-till-cancel"

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  "http://localhost:3000/orders?wallet=550e8400-e29b-41d4-a716-446655440000&status=open"
```

**Example Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "850e8400-e29b-41d4-a716-446655440003",
      "wallet": "550e8400-e29b-41d4-a716-446655440000",
      "market_id": "750e8400-e29b-41d4-a716-446655440002",
      "bid_asset": "550e8400-e29b-41d4-a716-446655440000",
      "ask_asset": "650e8400-e29b-41d4-a716-446655440001",
      "bid_amount": "1000.00",
      "ask_amount": "500.00",
      "price": "2.00",
      "filled_bid_amount": "0.00",
      "filled_ask_amount": "0.00",
      "mode": "good-till-cancel",
      "status": "open",
      "created_at": "2024-01-15T10:00:00",
      "filled_at": null,
      "cancelled_at": null,
      "expires_at": null,
      "order_type": "limit"
    }
  ],
  "error": null
}
```

---

## Time Series API

### GET /time-series/history

Get historical time series data for a market/asset pair.

**Query Parameters:**
- `market` (required): Market UUID
- `asset` (required): Asset UUID
- `duration_secs` (required): Duration in seconds (numeric string)
- `interval` (required): "1min", "5min", "15min", "30min", "1hr", "4hr", "1day", or "1week"

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  "http://localhost:3000/time-series/history?market=750e8400-e29b-41d4-a716-446655440002&asset=550e8400-e29b-41d4-a716-446655440000&duration_secs=86400&interval=1hr"
```

**Example Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "950e8400-e29b-41d4-a716-446655440004",
      "market_id": "750e8400-e29b-41d4-a716-446655440002",
      "asset": "550e8400-e29b-41d4-a716-446655440000",
      "open": "2.00",
      "high": "2.50",
      "low": "1.95",
      "close": "2.25",
      "volume": "10000.00",
      "created_at": "2024-01-15T10:00:00",
      "start_time": "2024-01-14T12:00:00",
      "end_time": "2024-01-14T13:00:00",
      "interval": "1hr",
      "data_provider_type": "order_book",
      "data_provider": "cradle-market"
    }
  ],
  "error": null
}
```

---

## Lending Pools API

### GET /pools/{id}

Get a lending pool by UUID.

**Parameters:**
- `id` (path, required): Lending pool UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/pools/a50e8400-e29b-41d4-a716-446655440005
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "a50e8400-e29b-41d4-a716-446655440005",
    "pool_address": "0.0.987654",
    "pool_contract_id": "0x1a2b3c4d5e6f",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440000",
    "loan_to_value": "0.75",
    "base_rate": "0.02",
    "slope1": "0.05",
    "slope2": "0.20",
    "liquidation_threshold": "0.85",
    "liquidation_discount": "0.10",
    "reserve_factor": "0.10",
    "name": "USDC Lending Pool",
    "title": "Stable Coin Lending",
    "description": "A lending pool for USDC stable coin",
    "created_at": "2024-01-01T07:00:00",
    "updated_at": "2024-01-15T10:00:00"
  },
  "error": null
}
```

---

### GET /pools/name/{name}

Get lending pool by name.

**Parameters:**
- `name` (path, required): Pool name

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/pools/name/USDC%20Lending%20Pool
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "a50e8400-e29b-41d4-a716-446655440005",
    "pool_address": "0.0.987654",
    "pool_contract_id": "0x1a2b3c4d5e6f",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440000",
    "loan_to_value": "0.75",
    "base_rate": "0.02",
    "slope1": "0.05",
    "slope2": "0.20",
    "liquidation_threshold": "0.85",
    "liquidation_discount": "0.10",
    "reserve_factor": "0.10",
    "name": "USDC Lending Pool",
    "title": "Stable Coin Lending",
    "description": "A lending pool for USDC stable coin",
    "created_at": "2024-01-01T07:00:00",
    "updated_at": "2024-01-15T10:00:00"
  },
  "error": null
}
```

---

### GET /pools/address/{address}

Get lending pool by contract address.

**Parameters:**
- `address` (path, required): Pool contract address

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/pools/address/0.0.987654
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "a50e8400-e29b-41d4-a716-446655440005",
    "pool_address": "0.0.987654",
    "pool_contract_id": "0x1a2b3c4d5e6f",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440000",
    "loan_to_value": "0.75",
    "base_rate": "0.02",
    "slope1": "0.05",
    "slope2": "0.20",
    "liquidation_threshold": "0.85",
    "liquidation_discount": "0.10",
    "reserve_factor": "0.10",
    "name": "USDC Lending Pool",
    "title": "Stable Coin Lending",
    "description": "A lending pool for USDC stable coin",
    "created_at": "2024-01-01T07:00:00",
    "updated_at": "2024-01-15T10:00:00"
  },
  "error": null
}
```

---

### GET /pools/{id}/snapshot

Get the latest snapshot for a lending pool.

**Parameters:**
- `id` (path, required): Lending pool UUID

**Example Request:**

```bash
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:3000/pools/a50e8400-e29b-41d4-a716-446655440005/snapshot
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "b50e8400-e29b-41d4-a716-446655440006",
    "lending_pool_id": "a50e8400-e29b-41d4-a716-446655440005",
    "total_supply": "1000000.00",
    "total_borrow": "750000.00",
    "available_liquidity": "250000.00",
    "utilization_rate": "0.75",
    "supply_apy": "0.05",
    "borrow_apy": "0.08",
    "created_at": "2024-01-15T10:00:00"
  },
  "error": null
}
```

---

## Mutations API (POST /process)

### POST /process

Process mutations/state-changing operations via the unified ActionRouter.

**Request Format:**

The request body must be a JSON object with a single top-level key representing the ActionRouterInput variant. The nested structure depends on the specific operation.

**Authentication:** Required (Bearer token)

---

### Accounts Mutations

#### Create Account

**Endpoint:** `POST /process`

**Request Body:**

```json
{
  "Accounts": {
    "CreateAccount": {
      "linked_account_id": "external-account-123",
      "account_type": "retail",
      "status": "unverified"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Accounts": {
      "CreateAccount": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "wallet_id": "660e8400-e29b-41d4-a716-446655440001"
      }
    }
  },
  "error": null
}
```

---

#### Create Account Wallet

**Request Body:**

```json
{
  "Accounts": {
    "CreateAccountWallet": {
      "cradle_account_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "active"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Accounts": {
      "CreateAccountWallet": {
        "id": "660e8400-e29b-41d4-a716-446655440001"
      }
    }
  },
  "error": null
}
```

---

#### Update Account Status

**Request Body:**

```json
{
  "Accounts": {
    "UpdateAccountStatus": {
      "cradle_account_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "verified"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Accounts": {
      "UpdateAccountStatus": null
    }
  },
  "error": null
}
```

---

#### Withdraw Tokens

**Request Body:**

```json
{
  "Accounts": {
    "WithdrawTokens": {
      "withdrawal_type": "crypto",
      "to": "0x1234567890abcdef",
      "amount": "1000.00",
      "token": "0.0.12345",
      "from": "660e8400-e29b-41d4-a716-446655440001"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Accounts": {
      "WithdrawTokens": null
    }
  },
  "error": null
}
```

---

### Asset Mutations

#### Create New Asset

**Request Body:**

```json
{
  "AssetBook": {
    "CreateNewAsset": {
      "asset_type": "stablecoin",
      "name": "USD Coin",
      "symbol": "USDC",
      "decimals": 6,
      "icon": "https://example.com/usdc.png"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "AssetBook": {
      "CreateNewAsset": "550e8400-e29b-41d4-a716-446655440000"
    }
  },
  "error": null
}
```

---

#### Create Existing Asset

**Request Body:**

```json
{
  "AssetBook": {
    "CreateExistingAsset": {
      "asset_manager": "0x1234567890abcdef",
      "token": "0.0.12345",
      "asset_type": "native",
      "name": "Hedera",
      "symbol": "HBAR",
      "decimals": 8,
      "icon": "https://example.com/hbar.png"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "AssetBook": {
      "CreateExistingAsset": "550e8400-e29b-41d4-a716-446655440000"
    }
  },
  "error": null
}
```

---

### Market Mutations

#### Create Market

**Request Body:**

```json
{
  "Markets": {
    "CreateMarket": {
      "name": "HBAR/USDC",
      "description": "Hedera to USD Coin trading pair",
      "icon": "https://example.com/hbar-usdc.png",
      "asset_one": "550e8400-e29b-41d4-a716-446655440000",
      "asset_two": "650e8400-e29b-41d4-a716-446655440001",
      "market_type": "spot",
      "market_status": "active",
      "market_regulation": "regulated"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Markets": {
      "CreateMarket": "750e8400-e29b-41d4-a716-446655440002"
    }
  },
  "error": null
}
```

---

#### Update Market Status

**Request Body:**

```json
{
  "Markets": {
    "UpdateMarketStatus": {
      "market_id": "750e8400-e29b-41d4-a716-446655440002",
      "status": "suspended"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Markets": {
      "UpdateMarketStatus": null
    }
  },
  "error": null
}
```

---

### Order Mutations

#### Place Order

**Request Body:**

```json
{
  "OrderBook": {
    "PlaceOrder": {
      "wallet": "550e8400-e29b-41d4-a716-446655440000",
      "market_id": "750e8400-e29b-41d4-a716-446655440002",
      "bid_asset": "550e8400-e29b-41d4-a716-446655440000",
      "ask_asset": "650e8400-e29b-41d4-a716-446655440001",
      "bid_amount": "1000.00",
      "ask_amount": "500.00",
      "price": "2.00",
      "mode": "fill-or-kill",
      "order_type": "limit"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "OrderBook": {
      "PlaceOrder": {
        "id": "850e8400-e29b-41d4-a716-446655440003",
        "status": "partial",
        "bid_amount_filled": "500.00",
        "ask_amount_filled": "250.00",
        "matched_trades": ["950e8400-e29b-41d4-a716-446655440004"]
      }
    }
  },
  "error": null
}
```

---

#### Cancel Order

**Request Body:**

```json
{
  "OrderBook": {
    "CancelOrder": "850e8400-e29b-41d4-a716-446655440003"
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "OrderBook": {
      "CancelOrder": null
    }
  },
  "error": null
}
```

---

### Time Series Mutations

#### Add Record

**Request Body:**

```json
{
  "MarketTimeSeries": {
    "AddRecord": {
      "market_id": "750e8400-e29b-41d4-a716-446655440002",
      "asset": "550e8400-e29b-41d4-a716-446655440000",
      "open": "2.00",
      "high": "2.50",
      "low": "1.95",
      "close": "2.25",
      "volume": "10000.00",
      "start_time": "2024-01-01T12:00:00",
      "end_time": "2024-01-01T13:00:00",
      "interval": "1hr",
      "data_provider_type": "order_book",
      "data_provider": "cradle-market"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "MarketTimeSeries": {
      "AddRecord": "950e8400-e29b-41d4-a716-446655440004"
    }
  },
  "error": null
}
```

---

### Lending Pool Mutations

#### Create Lending Pool

**Request Body:**

```json
{
  "Pool": {
    "CreateLendingPool": {
      "pool_address": "0.0.987654",
      "pool_contract_id": "0x1a2b3c4d5e6f",
      "reserve_asset": "550e8400-e29b-41d4-a716-446655440000",
      "loan_to_value": "0.75",
      "base_rate": "0.02",
      "slope1": "0.05",
      "slope2": "0.20",
      "liquidation_threshold": "0.85",
      "liquidation_discount": "0.10",
      "reserve_factor": "0.10",
      "name": "USDC Lending Pool",
      "title": "Stable Coin Lending",
      "description": "A lending pool for USDC stable coin"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Pool": {
      "CreateLendingPool": "a50e8400-e29b-41d4-a716-446655440005"
    }
  },
  "error": null
}
```

---

#### Supply Liquidity

**Request Body:**

```json
{
  "Pool": {
    "SupplyLiquidity": {
      "wallet": "550e8400-e29b-41d4-a716-446655440000",
      "pool": "a50e8400-e29b-41d4-a716-446655440005",
      "amount": 10000
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Pool": {
      "SupplyLiquidity": "b50e8400-e29b-41d4-a716-446655440006"
    }
  },
  "error": null
}
```

---

#### Borrow Asset

**Request Body:**

```json
{
  "Pool": {
    "BorrowAsset": {
      "wallet": "550e8400-e29b-41d4-a716-446655440000",
      "pool": "a50e8400-e29b-41d4-a716-446655440005",
      "amount": 5000,
      "collateral": "650e8400-e29b-41d4-a716-446655440001"
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Pool": {
      "BorrowAsset": "c50e8400-e29b-41d4-a716-446655440007"
    }
  },
  "error": null
}
```

---

#### Repay Borrow

**Request Body:**

```json
{
  "Pool": {
    "RepayBorrow": {
      "wallet": "550e8400-e29b-41d4-a716-446655440000",
      "loan": "c50e8400-e29b-41d4-a716-446655440007",
      "amount": 5500
    }
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "Pool": {
      "RepayBorrow": null
    }
  },
  "error": null
}
```

---

## Enum Reference

### Account Enums

| Enum | Values |
|------|--------|
| `CradleAccountType` | `retail`, `institutional` |
| `CradleAccountStatus` | `unverified`, `verified`, `suspended`, `closed` |
| `CradleWalletStatus` | `active`, `inactive`, `suspended` |
| `WithdrawalType` | `fiat`, `crypto` |

### Asset Enums

| Enum | Values |
|------|--------|
| `AssetType` | `bridged`, `native`, `yield_breaking`, `chain_native`, `stablecoin`, `volatile` |

### Market Enums

| Enum | Values |
|------|--------|
| `MarketStatus` | `active`, `inactive`, `suspended` |
| `MarketType` | `spot`, `derivative`, `futures` |
| `MarketRegulation` | `regulated`, `unregulated` |

### Order Enums

| Enum | Values |
|------|--------|
| `OrderStatus` | `open`, `closed`, `cancelled` |
| `OrderType` | `limit`, `market` |
| `FillMode` | `fill-or-kill`, `immediate-or-cancel`, `good-till-cancel` |
| `OrderFillStatus` | `partial`, `filled`, `cancelled` |

### Time Series Enums

| Enum | Values |
|------|--------|
| `TimeSeriesInterval` | `1min`, `5min`, `15min`, `30min`, `1hr`, `4hr`, `1day`, `1week` |
| `DataProviderType` | `order_book`, `exchange`, `aggregated` |

### Lending Pool Enums

| Enum | Values |
|------|--------|
| `LoanStatus` | `active`, `repaid`, `liquidated` |
| `PoolTransactionType` | `supply`, `withdraw` |

---

## Error Handling

### Common Error Responses

**Invalid Request (400):**

```json
{
  "success": false,
  "data": null,
  "error": "Invalid market ID format"
}
```

**Unauthorized (401):**

```json
{
  "success": false,
  "data": null,
  "error": "Invalid authentication token"
}
```

**Not Found (404):**

```json
{
  "success": false,
  "data": null,
  "error": "Account not found"
}
```

**Database Error (500):**

```json
{
  "success": false,
  "data": null,
  "error": "Action processing failed: database connection error"
}
```

---

## Rate Limiting

Currently, no rate limiting is implemented. This may be added in future versions.

---

## Changelog

### Version 1.0 (Current)

- Initial API release
- 23 GET endpoints for querying
- 1 POST endpoint for mutations via ActionRouter
- Bearer token authentication
- Comprehensive error handling
- All processor inputs/outputs support Serialize/Deserialize

