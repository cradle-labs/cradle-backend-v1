# Cradle Backend API Documentation

## Lending Pools & Loans API Reference

Complete API documentation for all lending pool, loan, repayment, and liquidation endpoints with exact request/response structures.

---

## Table of Contents

1. [Lending Pool Endpoints](#lending-pool-endpoints)
2. [Loan Query Endpoints](#loan-query-endpoints)
3. [Loan Repayment Endpoints](#loan-repayment-endpoints)
4. [Loan Liquidation Endpoints](#loan-liquidation-endpoints)
5. [Asset Lending Pool Contract Getters](#asset-lending-pool-contract-getters)
6. [Response Format](#response-format)
7. [Error Handling](#error-handling)

---

## Lending Pool Endpoints

### 1. Get All Lending Pools

**Route:** `GET /pools`

**Description:** Retrieve all lending pools from the database.

**Request:**
```
GET /pools
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "pool_address": "0x1234567890123456789012345678901234567890",
      "pool_contract_id": "0.0.123456",
      "reserve_asset": "550e8400-e29b-41d4-a716-446655440001",
      "loan_to_value": "0.8",
      "base_rate": "0.02",
      "slope1": "0.04",
      "slope2": "0.06",
      "liquidation_threshold": "0.85",
      "liquidation_discount": "0.05",
      "reserve_factor": "0.1",
      "name": "USDC Lending Pool",
      "title": "USDC Pool",
      "description": "Pool for USDC lending",
      "created_at": "2024-01-15T10:30:00",
      "updated_at": "2024-01-15T10:30:00"
    }
  ]
}
```

**Fields:**
- `id`: UUID - Unique pool identifier
- `pool_address`: String - Ethereum/Hedera contract address
- `pool_contract_id`: String - Hedera contract ID (0.0.xxx format)
- `reserve_asset`: UUID - Asset ID used as reserve
- `loan_to_value`: BigDecimal - LTV ratio (0-1)
- `base_rate`: BigDecimal - Base interest rate
- `slope1`: BigDecimal - First slope of interest rate model
- `slope2`: BigDecimal - Second slope of interest rate model
- `liquidation_threshold`: BigDecimal - Threshold for liquidation
- `liquidation_discount`: BigDecimal - Discount applied during liquidation
- `reserve_factor`: BigDecimal - Percentage of interest reserved
- `name`: String (Optional) - Pool name
- `title`: String (Optional) - Pool display title
- `description`: String (Optional) - Pool description
- `created_at`: Timestamp - Creation timestamp
- `updated_at`: Timestamp - Last update timestamp

---

### 2. Get Lending Pool by ID

**Route:** `GET /pools/{id}`

**Description:** Retrieve a specific lending pool by UUID.

**Request:**
```
GET /pools/550e8400-e29b-41d4-a716-446655440000
```

**Path Parameters:**
- `id` (UUID, required): Pool UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "pool_address": "0x1234567890123456789012345678901234567890",
    "pool_contract_id": "0.0.123456",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440001",
    "loan_to_value": "0.8",
    "base_rate": "0.02",
    "slope1": "0.04",
    "slope2": "0.06",
    "liquidation_threshold": "0.85",
    "liquidation_discount": "0.05",
    "reserve_factor": "0.1",
    "name": "USDC Lending Pool",
    "title": "USDC Pool",
    "description": "Pool for USDC lending",
    "created_at": "2024-01-15T10:30:00",
    "updated_at": "2024-01-15T10:30:00"
  }
}
```

**Errors:**
- `400 Bad Request`: Invalid pool ID format
- `404 Not Found`: Lending pool not found

---

### 3. Get Lending Pool by Name

**Route:** `GET /pools/name/{name}`

**Description:** Retrieve a lending pool by its name.

**Request:**
```
GET /pools/name/USDC%20Lending%20Pool
```

**Path Parameters:**
- `name` (String, required): Pool name (URL encoded)

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "pool_address": "0x1234567890123456789012345678901234567890",
    "pool_contract_id": "0.0.123456",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440001",
    "loan_to_value": "0.8",
    "base_rate": "0.02",
    "slope1": "0.04",
    "slope2": "0.06",
    "liquidation_threshold": "0.85",
    "liquidation_discount": "0.05",
    "reserve_factor": "0.1",
    "name": "USDC Lending Pool",
    "title": "USDC Pool",
    "description": "Pool for USDC lending",
    "created_at": "2024-01-15T10:30:00",
    "updated_at": "2024-01-15T10:30:00"
  }
}
```

**Errors:**
- `404 Not Found`: Lending pool not found

---

### 4. Get Lending Pool by Address

**Route:** `GET /pools/address/{address}`

**Description:** Retrieve a lending pool by its contract address.

**Request:**
```
GET /pools/address/0x1234567890123456789012345678901234567890
```

**Path Parameters:**
- `address` (String, required): Pool contract address

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "pool_address": "0x1234567890123456789012345678901234567890",
    "pool_contract_id": "0.0.123456",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440001",
    "loan_to_value": "0.8",
    "base_rate": "0.02",
    "slope1": "0.04",
    "slope2": "0.06",
    "liquidation_threshold": "0.85",
    "liquidation_discount": "0.05",
    "reserve_factor": "0.1",
    "name": "USDC Lending Pool",
    "title": "USDC Pool",
    "description": "Pool for USDC lending",
    "created_at": "2024-01-15T10:30:00",
    "updated_at": "2024-01-15T10:30:00"
  }
}
```

**Errors:**
- `404 Not Found`: Lending pool not found

---

### 5. Get Pool Snapshot

**Route:** `GET /pools/{id}/snapshot`

**Description:** Retrieve the latest snapshot (metrics) for a lending pool.

**Request:**
```
GET /pools/550e8400-e29b-41d4-a716-446655440000/snapshot
```

**Path Parameters:**
- `id` (UUID, required): Pool UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440000",
    "lending_pool_id": "550e8400-e29b-41d4-a716-446655440000",
    "total_supply": "1000000.5",
    "total_borrow": "750000.25",
    "available_liquidity": "250000.25",
    "utilization_rate": "0.75",
    "supply_apy": "0.045",
    "borrow_apy": "0.065",
    "created_at": "2024-01-15T10:30:00"
  }
}
```

**Fields:**
- `id`: UUID - Snapshot ID
- `lending_pool_id`: UUID - Associated pool ID
- `total_supply`: BigDecimal - Total assets supplied to pool
- `total_borrow`: BigDecimal - Total assets borrowed from pool
- `available_liquidity`: BigDecimal - Available liquidity for borrowing
- `utilization_rate`: BigDecimal - Utilization rate (0-1)
- `supply_apy`: BigDecimal - Annual Percentage Yield for suppliers
- `borrow_apy`: BigDecimal - Annual Percentage Yield for borrowers
- `created_at`: Timestamp - Snapshot creation time

**Errors:**
- `400 Bad Request`: Invalid pool ID format
- `404 Not Found`: Pool or snapshot not found

---

## Loan Query Endpoints

### 1. Get All Loans

**Route:** `GET /loans`

**Description:** Retrieve all loans from the database.

**Request:**
```
GET /loans
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "account_id": "880e8400-e29b-41d4-a716-446655440000",
      "wallet_id": "990e8400-e29b-41d4-a716-446655440000",
      "pool": "550e8400-e29b-41d4-a716-446655440000",
      "borrow_index": "1.05",
      "principal_amount": "10000.00",
      "created_at": "2024-01-10T08:15:00",
      "status": "active",
      "transaction": "0x1234567890abcdef"
    }
  ]
}
```

**Fields:**
- `id`: UUID - Unique loan identifier
- `account_id`: UUID - Associated account ID
- `wallet_id`: UUID - Associated wallet ID
- `pool`: UUID - Lending pool ID
- `borrow_index`: BigDecimal - Borrow index at loan creation
- `principal_amount`: BigDecimal - Principal amount borrowed
- `created_at`: Timestamp - Loan creation timestamp
- `status`: String - Loan status ("active", "repaid", or "liquidated")
- `transaction`: String (Optional) - Blockchain transaction hash

---

### 2. Get Loans by Pool

**Route:** `GET /loans/pool/{id}`

**Description:** Retrieve all loans associated with a specific lending pool.

**Request:**
```
GET /loans/pool/550e8400-e29b-41d4-a716-446655440000
```

**Path Parameters:**
- `id` (UUID, required): Pool UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "account_id": "880e8400-e29b-41d4-a716-446655440000",
      "wallet_id": "990e8400-e29b-41d4-a716-446655440000",
      "pool": "550e8400-e29b-41d4-a716-446655440000",
      "borrow_index": "1.05",
      "principal_amount": "10000.00",
      "created_at": "2024-01-10T08:15:00",
      "status": "active",
      "transaction": "0x1234567890abcdef"
    }
  ]
}
```

**Errors:**
- `400 Bad Request`: Invalid pool ID format

---

### 3. Get Loans by Wallet

**Route:** `GET /loans/wallet/{id}`

**Description:** Retrieve all loans associated with a specific wallet.

**Request:**
```
GET /loans/wallet/990e8400-e29b-41d4-a716-446655440000
```

**Path Parameters:**
- `id` (UUID, required): Wallet UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "account_id": "880e8400-e29b-41d4-a716-446655440000",
      "wallet_id": "990e8400-e29b-41d4-a716-446655440000",
      "pool": "550e8400-e29b-41d4-a716-446655440000",
      "borrow_index": "1.05",
      "principal_amount": "10000.00",
      "created_at": "2024-01-10T08:15:00",
      "status": "active",
      "transaction": "0x1234567890abcdef"
    }
  ]
}
```

**Errors:**
- `400 Bad Request`: Invalid wallet ID format

---

### 4. Get Loans by Status

**Route:** `GET /loans/status/{status}`

**Description:** Retrieve all loans filtered by status.

**Request:**
```
GET /loans/status/active
```

**Path Parameters:**
- `status` (String, required): Loan status - Must be one of: `active`, `repaid`, `liquidated` (case-insensitive)

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "account_id": "880e8400-e29b-41d4-a716-446655440000",
      "wallet_id": "990e8400-e29b-41d4-a716-446655440000",
      "pool": "550e8400-e29b-41d4-a716-446655440000",
      "borrow_index": "1.05",
      "principal_amount": "10000.00",
      "created_at": "2024-01-10T08:15:00",
      "status": "active",
      "transaction": "0x1234567890abcdef"
    }
  ]
}
```

**Errors:**
- `400 Bad Request`: Invalid loan status. Valid values: active, repaid, liquidated

---

## Loan Repayment Endpoints

### 1. Get All Repayments

**Route:** `GET /loan-repayments`

**Description:** Retrieve all loan repayment records, sorted by repayment date (newest first).

**Request:**
```
GET /loan-repayments
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "aa0e8400-e29b-41d4-a716-446655440000",
      "loan_id": "770e8400-e29b-41d4-a716-446655440000",
      "repayment_amount": "2500.50",
      "repayment_date": "2024-01-14T15:45:00",
      "transaction": "0x9876543210abcdef"
    }
  ]
}
```

**Fields:**
- `id`: UUID - Unique repayment record ID
- `loan_id`: UUID - Associated loan ID
- `repayment_amount`: BigDecimal - Amount repaid
- `repayment_date`: Timestamp - Date of repayment
- `transaction`: String (Optional) - Blockchain transaction hash

---

### 2. Get Repayments by Loan

**Route:** `GET /loan-repayments/loan/{id}`

**Description:** Retrieve all repayment records for a specific loan, sorted by repayment date (newest first).

**Request:**
```
GET /loan-repayments/loan/770e8400-e29b-41d4-a716-446655440000
```

**Path Parameters:**
- `id` (UUID, required): Loan UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "aa0e8400-e29b-41d4-a716-446655440000",
      "loan_id": "770e8400-e29b-41d4-a716-446655440000",
      "repayment_amount": "2500.50",
      "repayment_date": "2024-01-14T15:45:00",
      "transaction": "0x9876543210abcdef"
    },
    {
      "id": "bb0e8400-e29b-41d4-a716-446655440000",
      "loan_id": "770e8400-e29b-41d4-a716-446655440000",
      "repayment_amount": "2500.50",
      "repayment_date": "2024-01-13T10:20:00",
      "transaction": "0x8765432109abcdef"
    }
  ]
}
```

**Errors:**
- `400 Bad Request`: Invalid loan ID format

---

## Loan Liquidation Endpoints

### 1. Get All Liquidations

**Route:** `GET /loan-liquidations`

**Description:** Retrieve all loan liquidation records, sorted by liquidation date (newest first).

**Request:**
```
GET /loan-liquidations
```

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "cc0e8400-e29b-41d4-a716-446655440000",
      "loan_id": "770e8400-e29b-41d4-a716-446655440000",
      "liquidator_wallet_id": "dd0e8400-e29b-41d4-a716-446655440000",
      "liquidation_amount": "5000.00",
      "liquidation_date": "2024-01-12T12:00:00",
      "transaction": "0x1111111111111111"
    }
  ]
}
```

**Fields:**
- `id`: UUID - Unique liquidation record ID
- `loan_id`: UUID - Associated loan ID
- `liquidator_wallet_id`: UUID - Wallet ID of the liquidator
- `liquidation_amount`: BigDecimal - Amount liquidated
- `liquidation_date`: Timestamp - Date of liquidation
- `transaction`: String (Optional) - Blockchain transaction hash

---

### 2. Get Liquidations by Loan

**Route:** `GET /loan-liquidations/loan/{id}`

**Description:** Retrieve all liquidation records for a specific loan, sorted by liquidation date (newest first).

**Request:**
```
GET /loan-liquidations/loan/770e8400-e29b-41d4-a716-446655440000
```

**Path Parameters:**
- `id` (UUID, required): Loan UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": [
    {
      "id": "cc0e8400-e29b-41d4-a716-446655440000",
      "loan_id": "770e8400-e29b-41d4-a716-446655440000",
      "liquidator_wallet_id": "dd0e8400-e29b-41d4-a716-446655440000",
      "liquidation_amount": "5000.00",
      "liquidation_date": "2024-01-12T12:00:00",
      "transaction": "0x1111111111111111"
    }
  ]
}
```

**Errors:**
- `400 Bad Request`: Invalid loan ID format

---

## Asset Lending Pool Contract Getters

### 1. Get Interest Rates

**Route:** `GET /pools/{id}/interest-rates`

**Description:** Retrieve the interest rate configuration for a lending pool, including the two-slope interest rate model.

**Request:**
```
GET /pools/550e8400-e29b-41d4-a716-446655440000/interest-rates
```

**Path Parameters:**
- `id` (UUID, required): Pool UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "pool_id": "550e8400-e29b-41d4-a716-446655440000",
    "base_rate": "0.02",
    "slope1": "0.04",
    "slope2": "0.06",
    "reserve_factor": "0.1",
    "interest_rate_model": {
      "description": "Two-slope interest rate model",
      "slope1_threshold": "Kink point where slope changes",
      "slope1_rate": "0.04",
      "slope2_rate": "0.06"
    }
  }
}
```

**Fields:**
- `pool_id`: UUID - Pool identifier
- `base_rate`: BigDecimal - Base interest rate
- `slope1`: BigDecimal - First slope rate (below kink point)
- `slope2`: BigDecimal - Second slope rate (above kink point)
- `reserve_factor`: BigDecimal - Percentage of interest reserved
- `interest_rate_model.description`: String - Model type description
- `interest_rate_model.slope1_threshold`: String - Description of kink point
- `interest_rate_model.slope1_rate`: BigDecimal - First slope rate value
- `interest_rate_model.slope2_rate`: BigDecimal - Second slope rate value

**Errors:**
- `400 Bad Request`: Invalid pool ID format
- `404 Not Found`: Lending pool not found

---

### 2. Get Collateral Info

**Route:** `GET /pools/{id}/collateral-info`

**Description:** Retrieve the collateral configuration and risk parameters for a lending pool.

**Request:**
```
GET /pools/550e8400-e29b-41d4-a716-446655440000/collateral-info
```

**Path Parameters:**
- `id` (UUID, required): Pool UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "pool_id": "550e8400-e29b-41d4-a716-446655440000",
    "loan_to_value": "0.8",
    "liquidation_threshold": "0.85",
    "liquidation_discount": "0.05",
    "risk_parameters": {
      "ltv": "0.8",
      "liquidation_threshold": "0.85",
      "liquidation_penalty": "0.05"
    }
  }
}
```

**Fields:**
- `pool_id`: UUID - Pool identifier
- `loan_to_value`: BigDecimal - LTV ratio (maximum loan amount relative to collateral)
- `liquidation_threshold`: BigDecimal - Health factor threshold for liquidation
- `liquidation_discount`: BigDecimal - Discount applied to liquidator
- `risk_parameters.ltv`: BigDecimal - Loan-to-Value ratio
- `risk_parameters.liquidation_threshold`: BigDecimal - Liquidation threshold
- `risk_parameters.liquidation_penalty`: BigDecimal - Liquidation penalty percentage

**Errors:**
- `400 Bad Request`: Invalid pool ID format
- `404 Not Found`: Lending pool not found

---

### 3. Get Pool Statistics

**Route:** `GET /pools/{id}/pool-stats`

**Description:** Retrieve comprehensive statistics and metrics for a lending pool.

**Request:**
```
GET /pools/550e8400-e29b-41d4-a716-446655440000/pool-stats
```

**Path Parameters:**
- `id` (UUID, required): Pool UUID

**Response:** `200 OK` (With Snapshot)
```json
{
  "success": true,
  "data": {
    "pool_id": "550e8400-e29b-41d4-a716-446655440000",
    "pool_name": "USDC Lending Pool",
    "pool_address": "0x1234567890123456789012345678901234567890",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440001",
    "metrics": {
      "total_supply": "1000000.5",
      "total_borrow": "750000.25",
      "available_liquidity": "250000.25",
      "utilization_rate": "0.75",
      "supply_apy": "0.045",
      "borrow_apy": "0.065"
    },
    "last_updated": "2024-01-15T10:30:00",
    "rate_configuration": {
      "base_rate": "0.02",
      "slope1": "0.04",
      "slope2": "0.06"
    }
  }
}
```

**Response:** `200 OK` (Without Snapshot)
```json
{
  "success": true,
  "data": {
    "pool_id": "550e8400-e29b-41d4-a716-446655440000",
    "pool_name": "USDC Lending Pool",
    "pool_address": "0x1234567890123456789012345678901234567890",
    "reserve_asset": "550e8400-e29b-41d4-a716-446655440001",
    "metrics": {
      "total_supply": null,
      "total_borrow": null,
      "available_liquidity": null,
      "utilization_rate": null,
      "supply_apy": null,
      "borrow_apy": null
    },
    "note": "No snapshots available yet",
    "rate_configuration": {
      "base_rate": "0.02",
      "slope1": "0.04",
      "slope2": "0.06"
    }
  }
}
```

**Fields:**
- `pool_id`: UUID - Pool identifier
- `pool_name`: String - Pool name
- `pool_address`: String - Contract address
- `reserve_asset`: UUID - Reserve asset ID
- `metrics.total_supply`: BigDecimal (or null) - Total assets supplied
- `metrics.total_borrow`: BigDecimal (or null) - Total assets borrowed
- `metrics.available_liquidity`: BigDecimal (or null) - Available liquidity
- `metrics.utilization_rate`: BigDecimal (or null) - Pool utilization (0-1)
- `metrics.supply_apy`: BigDecimal (or null) - Supply APY
- `metrics.borrow_apy`: BigDecimal (or null) - Borrow APY
- `last_updated`: Timestamp (optional) - Last snapshot update time
- `note`: String (optional) - Message when no snapshots exist
- `rate_configuration.base_rate`: BigDecimal - Base interest rate
- `rate_configuration.slope1`: BigDecimal - First slope rate
- `rate_configuration.slope2`: BigDecimal - Second slope rate

**Errors:**
- `400 Bad Request`: Invalid pool ID format
- `404 Not Found`: Lending pool not found

---

### 4. Get User Positions

**Route:** `GET /pools/{pool_id}/user-positions/{wallet_id}`

**Description:** Retrieve detailed borrow position and repayment history for a user in a specific pool.

**Request:**
```
GET /pools/550e8400-e29b-41d4-a716-446655440000/user-positions/990e8400-e29b-41d4-a716-446655440000
```

**Path Parameters:**
- `pool_id` (UUID, required): Pool UUID
- `wallet_id` (UUID, required): Wallet UUID

**Response:** `200 OK`
```json
{
  "success": true,
  "data": {
    "pool_id": "550e8400-e29b-41d4-a716-446655440000",
    "wallet_id": "990e8400-e29b-41d4-a716-446655440000",
    "borrow_position": {
      "active_loans_count": 2,
      "total_borrow_amount": "20000.00",
      "loans": [
        {
          "loan_id": "770e8400-e29b-41d4-a716-446655440000",
          "principal_amount": "10000.00",
          "status": "Active",
          "created_at": "2024-01-10T08:15:00"
        },
        {
          "loan_id": "770e8400-e29b-41d4-a716-446655440001",
          "principal_amount": "10000.00",
          "status": "Active",
          "created_at": "2024-01-12T14:20:00"
        }
      ]
    },
    "repayment_history": {
      "total_repaid": "5000.00",
      "repayment_count": 2,
      "recent_repayments": [
        {
          "repayment_amount": "2500.50",
          "repayment_date": "2024-01-14T15:45:00"
        },
        {
          "repayment_amount": "2499.50",
          "repayment_date": "2024-01-13T10:20:00"
        }
      ]
    }
  }
}
```

**Fields:**
- `pool_id`: UUID - Pool identifier
- `wallet_id`: UUID - Wallet identifier
- `borrow_position.active_loans_count`: Integer - Number of active loans
- `borrow_position.total_borrow_amount`: BigDecimal - Total amount borrowed (active loans only)
- `borrow_position.loans[]`: Array of loan objects
  - `loan_id`: UUID - Loan identifier
  - `principal_amount`: BigDecimal - Principal borrowed
  - `status`: String - Loan status ("Active", "Repaid", or "Liquidated")
  - `created_at`: Timestamp - Loan creation date
- `repayment_history.total_repaid`: BigDecimal - Total amount repaid across all loans
- `repayment_history.repayment_count`: Integer - Number of repayment transactions
- `repayment_history.recent_repayments[]`: Array of last 5 repayments
  - `repayment_amount`: BigDecimal - Repayment amount
  - `repayment_date`: Timestamp - Repayment date

**Errors:**
- `400 Bad Request`: Invalid pool ID or wallet ID format
- `404 Not Found`: Lending pool not found

---

## Response Format

### Success Response
All successful API responses follow this format:
```json
{
  "success": true,
  "data": {}
}
```

### Error Response
All error API responses follow this format:
```json
{
  "success": false,
  "error": "Error message"
}
```

---

## Error Handling

### Common HTTP Status Codes

| Status Code | Meaning | Scenario |
|------------|---------|----------|
| 200 | OK | Request successful |
| 400 | Bad Request | Invalid path parameter format (e.g., invalid UUID) |
| 404 | Not Found | Resource not found in database |
| 500 | Internal Server Error | Database connection error or serialization error |

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Invalid pool ID format" | UUID is malformed | Ensure UUID is in format: `550e8400-e29b-41d4-a716-446655440000` |
| "Invalid wallet ID format" | UUID is malformed | Ensure UUID is in valid format |
| "Invalid loan ID format" | UUID is malformed | Ensure UUID is in valid format |
| "Invalid loan status. Use: active, repaid, or liquidated" | Invalid status parameter | Use one of: `active`, `repaid`, `liquidated` (case-insensitive) |
| "Lending pool not found" | Pool doesn't exist | Check pool UUID is correct |
| "Failed to acquire database connection" | Database connection pool error | Try request again, check database status |
| "Database error" | Database query failed | Check data consistency in database |

---

## Data Type Reference

### Basic Types
- **UUID**: `550e8400-e29b-41d4-a716-446655440000`
- **String**: Plain text
- **Timestamp**: ISO 8601 format - `2024-01-15T10:30:00`
- **BigDecimal**: Decimal number as string - `"1000.50"`
- **Integer**: Whole number - `2`

### Enum Types
- **LoanStatus**: `active`, `repaid`, `liquidated`
- **PoolTransactionType**: `supply`, `withdraw`

---

## Rate Limiting & Caching

- No rate limiting currently implemented
- All endpoints perform fresh database queries (no caching)
- For high-frequency queries, consider implementing client-side caching

---

## Authentication

All endpoints require valid authentication credentials (implementation details depend on your auth middleware configuration).

---

**Last Updated:** 2024-01-15
**API Version:** 1.0
