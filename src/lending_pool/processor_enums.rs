use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::lending_pool::db_types::{CreateLendingPoolRecord, LendingPoolRecord, LendingPoolSnapShotRecord};

#[derive(Serialize,Deserialize, Debug, Clone )]
pub enum GetLendingPoolInput {
    ByName(String),
    ByAddress(String),
    ById(Uuid)
}

#[derive(Serialize,Deserialize, Debug, Clone )]
pub struct SupplyLiquidityInputArgs {
    pub wallet: Uuid,
    pub pool: Uuid,
    pub amount: u64
}

#[derive(Serialize,Deserialize, Debug, Clone )]
pub struct WithdrawLiquidityInputArgs {
    pub wallet: Uuid,
    pub pool: Uuid,
    pub amount: u64 // in yield asset
}

#[derive(Serialize, Deserialize, Debug, Clone )]
pub struct TakeLoanInputArgs {
    pub wallet: Uuid,
    pub pool:Uuid,
    pub amount: u64,
    pub collateral: Uuid
}

#[derive(Serialize, Deserialize, Debug, Clone )]
pub struct RepayLoanInputArgs {
    pub wallet: Uuid,
    pub loan: Uuid,
    pub amount: u64
}

#[derive(Serialize, Deserialize, Debug, Clone )]
pub struct LiquidatePositionInputArgs {
    pub wallet: Uuid,
    pub loan: Uuid,
    pub amount: u64
}

#[derive(Deserialize, Serialize)]
pub enum LendingPoolFunctionsInput {
    CreateLendingPool(CreateLendingPoolRecord),
    GetLendingPool(GetLendingPoolInput),
    CreateSnapShot(Uuid),
    GetSnapShot(Uuid),
    // supply liquidity
    SupplyLiquidity(SupplyLiquidityInputArgs),
    WithdrawLiquidity(WithdrawLiquidityInputArgs),
    // borrow asset
    BorrowAsset(TakeLoanInputArgs),
    RepayBorrow(RepayLoanInputArgs),
    LiquidatePosition(LiquidatePositionInputArgs) 
}

#[derive(Deserialize, Serialize)]
pub enum LendingPoolFunctionsOutput {
    CreateLendingPool(Uuid),
    GetLendingPool(LendingPoolRecord),
    CreateSnapShot(Uuid),
    GetSnapShot(LendingPoolSnapShotRecord),
    SupplyLiquidity(Uuid),
    WithdrawLiquidity(Uuid),
    BorrowAsset(Uuid),
    RepayBorrow(),
    LiquidatePosition()
}


