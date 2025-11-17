-- Your SQL goes here


create table if not exists CradleListedCompanies (
    id uuid primary key default uuid_generate_v4(),
    name text not null,
    description text not null,
    listed_at timestamp default now(),
    legal_documents text not null, -- link to legal documents
    -- TODO: Post MVP will need to decide on additional details to add for companies
    beneficiary_wallet uuid not null  references CradleWalletAccounts(id)
    -- TODO: figure out company ownership setup, and who can control the listed companies
);


create type listing_status as enum (
    'pending',
    'open',
    'closed',
    'paused',
    'cancelled'
);

create table if not exists CradleNativeListings (
    id uuid primary key default uuid_generate_v4(),
    listing_contract_id text not null unique,
    name text not null,
    description text not null,
    documents text not null, -- link to documents on ipfs or something
    company uuid not null references CradleListedCompanies(id),
    status listing_status not null default 'pending',
    created_at timestamp not null default now(),
    opened_at timestamp,
    stopped_at timestamp, -- for either close or cancelling
    listed_asset uuid not null references asset_book(id),
    purchase_with_asset uuid not null references asset_book(id),
    purchase_price numeric not null,
    max_supply numeric not null,
    treasury uuid not null references CradleWalletAccounts(id),
    shadow_asset uuid not null references asset_book(id)
    -- TODO: support for different types e.g equity backed, governance based additional info in docs
);
