# Cradle Backend Security Model

## Overview

This document outlines the security architecture of the Cradle platform. The system is designed with a trusted-middleware model where the Cradle Backend operates as a protected service layer, accessible only by authorized upstream servers, not directly by end-user clients.

## Trust Architecture

The current deployment model assumes a **Trusted Client** relationship between the Frontend Server (Next.js) and the Cradle Backend.

*   **Users** authenticate with the Frontend (via Clerk).
*   **Frontend** holds the custody of API secrets.
*   **Backend** trusts the Frontend to have performed user authentication and authorization.

### 1. Authentication Flow

End users do not interact directly with the Cradle Backend or the Blockchain. The Next.js frontend acts as the custodian of the user session and the bridge to the backend.

```mermaid
sequenceDiagram
    participant User
    participant Clerk as Clerk Auth
    participant Next as Next.js Server
    participant Backend as Cradle Backend
    participant Chain as Hedera Network

    User->>Next: Login Request
    Next->>Clerk: Validate Credentials
    Clerk-->>Next: Session Token (JWT)
    
    Note over Next: User Authenticated

    User->>Next: Perform Action (e.g., Trade)
    Note over Next: 1. Validate User Session<br/>2. Retrieve Backend API Secret<br/>3. Construct Payload
    
    Next->>Backend: POST /process (Header: x-api-key)
    Backend->>Backend: Validate Middleware Auth
    
    Backend->>Chain: Execute Transaction
    Chain-->>Backend: Receipt
    Backend-->>Next: Success Response
    Next-->>User: Update UI
```

### 2. Network Security Perimeter

The backend is isolated from public internet traffic. Access is restricted using network-level controls and application-level secrets.

```mermaid
graph TD
    subgraph Public_Internet ["Public Internet"]
        User[End User / Browser]
    end

    subgraph Secure_Perimeter ["Secure Perimeter (VPC/Cluster)"]
        subgraph DMZ_Zone ["DMZ"]
            Next[Next.js Frontend Server]
        end

        subgraph Private_Subnet ["Private Subnet"]
            Backend[Cradle Backend API]
            DB[(Postgres DB)]
        end
    end

    User -- HTTPS / WSS --> Next
    User -. X .- Backend
    
    Next -- "Authenticated (API Key)" --> Backend
    Backend -- SQL --> DB
```

## Security Controls

### Application Layer
*   **API Secret Authentication**: The backend enforces strict authentication via `x-api-key` headers. This key is stored securely on the Next.js server side environment and is never exposed to the client browser.
*   **Clerk Integration**: User identity management, password storage, and 2FA are offloaded to Clerk, reducing the attack surface on the Cradle infrastructure.
*   **Wallet Abstraction**: Users do not manage private keys for protocol interaction. The architecture implies a custodial or semi-custodial model where the system manages the complexity of blockchain interactions on behalf of the web2-style user.

### Future Considerations: Multi-Tenancy
*   **Current State**: The system currently operates in a single-tenant mode, serving only the official Cradle frontend.
*   **Development Roadmap**: Support for multi-tenancy (allowing third-party integrators to build on top of Cradle Backend) is under development. This will require:
    *   Granular API Key management per tenant.
    *   Scoped resource access (Data segregation by Tenant ID).
    *   Rate limiting per tenant.

## Data Privacy & Integrity
*   **Ledger Immutability**: Critical financial events are recorded in the `accounts_ledger` tables, which serve as the source of truth for user balances and specific audit trails.
*   **On-Chain Verification**: Settlements and asset movements are executed on the Hedera network, providing a public, immutable proof of transaction that can be reconciled against the internal database.
