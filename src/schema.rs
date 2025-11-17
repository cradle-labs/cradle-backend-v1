// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "asset_type"))]
    pub struct AssetType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "cradleaccountstatus"))]
    pub struct Cradleaccountstatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "cradleaccounttype"))]
    pub struct Cradleaccounttype;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "cradlewalletstatus"))]
    pub struct Cradlewalletstatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "data_provider_type"))]
    pub struct DataProviderType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "fill_mode"))]
    pub struct FillMode;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "listing_status"))]
    pub struct ListingStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "loan_status"))]
    pub struct LoanStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "market_regulation"))]
    pub struct MarketRegulation;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "market_status"))]
    pub struct MarketStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "market_type"))]
    pub struct MarketType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_status"))]
    pub struct OrderStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_type"))]
    pub struct OrderType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pool_transaction_type"))]
    pub struct PoolTransactionType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "settlement_status"))]
    pub struct SettlementStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "time_series_interval"))]
    pub struct TimeSeriesInterval;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_type"))]
    pub struct TransactionType;
}

diesel::table! {
    accountassetbook (id) {
        id -> Uuid,
        asset_id -> Uuid,
        account_id -> Uuid,
        associated -> Bool,
        kyced -> Bool,
        associated_at -> Nullable<Timestamp>,
        kyced_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionType;

    accountassetsledger (id) {
        id -> Uuid,
        timestamp -> Timestamp,
        transaction -> Nullable<Text>,
        from_address -> Text,
        to_address -> Text,
        asset -> Uuid,
        transaction_type -> TransactionType,
        amount -> Numeric,
        refference -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AssetType;

    asset_book (id) {
        id -> Uuid,
        asset_manager -> Text,
        token -> Text,
        created_at -> Timestamp,
        asset_type -> AssetType,
        name -> Text,
        symbol -> Text,
        decimals -> Int4,
        icon -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Cradleaccounttype;
    use super::sql_types::Cradleaccountstatus;

    cradleaccounts (id) {
        id -> Uuid,
        linked_account_id -> Text,
        created_at -> Timestamp,
        account_type -> Cradleaccounttype,
        status -> Cradleaccountstatus,
    }
}

diesel::table! {
    cradlelistedcompanies (id) {
        id -> Uuid,
        name -> Text,
        description -> Text,
        listed_at -> Nullable<Timestamp>,
        legal_documents -> Text,
        beneficiary_wallet -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ListingStatus;

    cradlenativelistings (id) {
        id -> Uuid,
        listing_contract_id -> Text,
        name -> Text,
        description -> Text,
        documents -> Text,
        company -> Uuid,
        status -> ListingStatus,
        created_at -> Timestamp,
        opened_at -> Nullable<Timestamp>,
        stopped_at -> Nullable<Timestamp>,
        listed_asset -> Uuid,
        purchase_with_asset -> Uuid,
        purchase_price -> Numeric,
        max_supply -> Numeric,
        treasury -> Uuid,
        shadow_asset -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Cradlewalletstatus;

    cradlewalletaccounts (id) {
        id -> Uuid,
        cradle_account_id -> Uuid,
        address -> Text,
        contract_id -> Text,
        created_at -> Timestamp,
        status -> Cradlewalletstatus,
    }
}

diesel::table! {
    kvstore (key) {
        key -> Text,
        value -> Nullable<Text>,
    }
}

diesel::table! {
    lendingpool (id) {
        id -> Uuid,
        pool_address -> Text,
        pool_contract_id -> Text,
        reserve_asset -> Uuid,
        loan_to_value -> Numeric,
        base_rate -> Numeric,
        slope1 -> Numeric,
        slope2 -> Numeric,
        liquidation_threshold -> Numeric,
        liquidation_discount -> Numeric,
        reserve_factor -> Numeric,
        name -> Nullable<Text>,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        yield_asset -> Uuid,
    }
}

diesel::table! {
    lendingpoolsnapshots (id) {
        id -> Uuid,
        lending_pool_id -> Uuid,
        total_supply -> Numeric,
        total_borrow -> Numeric,
        available_liquidity -> Numeric,
        utilization_rate -> Numeric,
        supply_apy -> Numeric,
        borrow_apy -> Numeric,
        created_at -> Timestamp,
    }
}

diesel::table! {
    loanliquidations (id) {
        id -> Uuid,
        loan_id -> Uuid,
        liquidator_wallet_id -> Uuid,
        liquidation_amount -> Numeric,
        liquidation_date -> Timestamp,
        transaction -> Nullable<Text>,
    }
}

diesel::table! {
    loanrepayments (id) {
        id -> Uuid,
        loan_id -> Uuid,
        repayment_amount -> Numeric,
        repayment_date -> Timestamp,
        transaction -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LoanStatus;

    loans (id) {
        id -> Uuid,
        account_id -> Uuid,
        wallet_id -> Uuid,
        pool -> Uuid,
        borrow_index -> Numeric,
        principal_amount -> Numeric,
        created_at -> Timestamp,
        status -> LoanStatus,
        transaction -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MarketType;
    use super::sql_types::MarketStatus;
    use super::sql_types::MarketRegulation;

    markets (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        icon -> Nullable<Text>,
        asset_one -> Uuid,
        asset_two -> Uuid,
        created_at -> Timestamp,
        market_type -> MarketType,
        market_status -> MarketStatus,
        market_regulation -> MarketRegulation,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TimeSeriesInterval;
    use super::sql_types::DataProviderType;

    markets_time_series (id) {
        id -> Uuid,
        market_id -> Uuid,
        asset -> Uuid,
        open -> Numeric,
        high -> Numeric,
        low -> Numeric,
        close -> Numeric,
        volume -> Numeric,
        created_at -> Timestamp,
        start_time -> Timestamp,
        end_time -> Timestamp,
        interval -> TimeSeriesInterval,
        data_provider_type -> DataProviderType,
        data_provider -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FillMode;
    use super::sql_types::OrderStatus;
    use super::sql_types::OrderType;

    orderbook (id) {
        id -> Uuid,
        wallet -> Uuid,
        market_id -> Uuid,
        bid_asset -> Uuid,
        ask_asset -> Uuid,
        bid_amount -> Numeric,
        ask_amount -> Numeric,
        price -> Numeric,
        filled_bid_amount -> Numeric,
        filled_ask_amount -> Numeric,
        mode -> FillMode,
        status -> OrderStatus,
        created_at -> Timestamp,
        filled_at -> Nullable<Timestamp>,
        cancelled_at -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
        order_type -> OrderType,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SettlementStatus;

    orderbooktrades (id) {
        id -> Uuid,
        maker_order_id -> Uuid,
        taker_order_id -> Uuid,
        maker_filled_amount -> Numeric,
        taker_filled_amount -> Numeric,
        settlement_tx -> Nullable<Text>,
        settlement_status -> SettlementStatus,
        created_at -> Timestamp,
        settled_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PoolTransactionType;

    pooltransactions (id) {
        id -> Uuid,
        pool_id -> Uuid,
        wallet_id -> Uuid,
        amount -> Numeric,
        supply_index -> Numeric,
        transaction_type -> PoolTransactionType,
        yield_token_amount -> Numeric,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        transaction -> Text,
    }
}

diesel::joinable!(accountassetbook -> asset_book (asset_id));
diesel::joinable!(accountassetbook -> cradlewalletaccounts (account_id));
diesel::joinable!(accountassetsledger -> asset_book (asset));
diesel::joinable!(cradlelistedcompanies -> cradlewalletaccounts (beneficiary_wallet));
diesel::joinable!(cradlenativelistings -> cradlelistedcompanies (company));
diesel::joinable!(cradlenativelistings -> cradlewalletaccounts (treasury));
diesel::joinable!(cradlewalletaccounts -> cradleaccounts (cradle_account_id));
diesel::joinable!(lendingpoolsnapshots -> lendingpool (lending_pool_id));
diesel::joinable!(loanliquidations -> cradlewalletaccounts (liquidator_wallet_id));
diesel::joinable!(loanliquidations -> loans (loan_id));
diesel::joinable!(loanrepayments -> loans (loan_id));
diesel::joinable!(loans -> cradleaccounts (account_id));
diesel::joinable!(loans -> cradlewalletaccounts (wallet_id));
diesel::joinable!(loans -> lendingpool (pool));
diesel::joinable!(markets_time_series -> asset_book (asset));
diesel::joinable!(markets_time_series -> markets (market_id));
diesel::joinable!(orderbook -> cradlewalletaccounts (wallet));
diesel::joinable!(orderbook -> markets (market_id));
diesel::joinable!(pooltransactions -> cradlewalletaccounts (wallet_id));
diesel::joinable!(pooltransactions -> lendingpool (pool_id));

diesel::allow_tables_to_appear_in_same_query!(
    accountassetbook,
    accountassetsledger,
    asset_book,
    cradleaccounts,
    cradlelistedcompanies,
    cradlenativelistings,
    cradlewalletaccounts,
    kvstore,
    lendingpool,
    lendingpoolsnapshots,
    loanliquidations,
    loanrepayments,
    loans,
    markets,
    markets_time_series,
    orderbook,
    orderbooktrades,
    pooltransactions,
);
