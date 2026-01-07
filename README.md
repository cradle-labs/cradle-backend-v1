# Cradle Backend V1

A high-performance, modular backend service written in Rust for the Cradle DeFi platform. It handles off-chain order matching, managing user accounts, lending protocol logic, and market data aggregation, while facilitating secure interaction with the Hedera hashgraph via the `contract-integrator`.

## 📚 Documentation

Detailed documentation for the system architecture and security model can be found in the following files:

*   **[Architecture Guide](ARCHITECTURE.md)**: Breakdown of the system modules, request flow, and processor patterns.
*   **[Security Model](SECURITY_MODEL.md)**: Overview of the authentication flow, trust architecture, and network security.

## ✨ Key Features

*   **Account Management**: Abstraction of crypto wallets for Web2-style user accounts.
*   **Trading Engine**: High-speed, off-chain Order Book with on-chain settlement.
*   **Lending Protocol**: Management of lending pools, borrowing, repayments, and liquidations.
*   **Market Data**: Real-time ticker updates via WebSockets and historical OHLCV data processing.
*   **Ledger System**: Double-entry bookkeeping system (`accounts_ledger`) for immutable transaction auditability.
*   **Fiat On-Ramp**: Integration with Ramper for seamless fiat-to-crypto conversion.

## 🛠 Tech Stack

*   **Runtime**: [Rust](https://www.rust-lang.org/) (Tokio Async Runtime)
*   **API Framework**: [Axum](https://github.com/tokio-rs/axum)
*   **Database**: PostgreSQL (via [Diesel ORM](https://diesel.rs/))
*   **Real-time**: Socket.io (via `socketioxide`)
*   **Blockchain**: Hedera (via internal `contract-integrator` crate)

## 🚀 Getting Started

### Prerequisites

*   [Rust 1.90+](https://www.rust-lang.org/tools/install)
*   [PostgreSQL](https://www.postgresql.org/)
*   [Diesel CLI](https://diesel.rs/guides/getting-started.html) (`cargo install diesel_cli --no-default-features --features postgres`)

### Installation

1.  **Clone the repository**
    ```bash
    git clone https://github.com/cradle-labs/cradle-backend-v1.git
    cd cradle-backend-v1
    ```

2.  **Environment Setup**
    Create a `.env` file in the root directory (refer to `.env.example` if available) and configure your database URL and API secrets.
    ```env
    DATABASE_URL=postgres://user:password@localhost/cradle_db
    API_SECRET_KEY=your_secure_key
    # ... other configurations
    ```

3.  **Database Migration**
    Apply the database schemas using Diesel.
    ```bash
    diesel setup
    diesel migration run
    ```

4.  **Build the Project**
    ```bash
    cargo build --release
    ```

### Running the Server

Start the main API server.
```bash
cargo run
```

Access the health check endpoint to verify the server is running:
```
GET /health
```

### Additional Tools

The workspace includes several binary utilities:

*   **Admin Dashboard**:
    ```bash
    cargo run --bin admin_dashboard
    ```
*   **Listings CLI**:
    ```bash
    cargo run --bin listings-cli
    ```

## 🔐 Security

This backend is designed to run inside a private network, accessible only by the trusted Frontend Server (Next.js) which handles user authentication via Clerk. Direct access by end-users is strictly prohibited. See [SECURITY_MODEL.md](SECURITY_MODEL.md) for details.

## 📄 License

Copyright &copy; Cradle Labs. All rights reserved.
