// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "cradleaccountstatus"))]
    pub struct Cradleaccountstatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "cradleaccounttype"))]
    pub struct Cradleaccounttype;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "cradlewalletstatus"))]
    pub struct Cradlewalletstatus;
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

diesel::joinable!(cradlewalletaccounts -> cradleaccounts (cradle_account_id));

diesel::allow_tables_to_appear_in_same_query!(cradleaccounts, cradlewalletaccounts,);
