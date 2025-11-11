use std::env;

use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::Utc;
use contract_integrator::utils::functions::cradle_account::TransferAssetArgs;
use contract_integrator::utils::functions::orderbook_settler::OrderBookSettlerFunctionOutput;
use contract_integrator::wallet::wallet::ActionWallet;
use diesel::prelude::*;
use crate::accounts::db_types::CradleWalletAccountRecord;
use crate::asset_book::db_types::AssetBookRecord;
use crate::order_book::db_types::{OrderBookRecord, OrderBookTradeRecord, OrderStatus, SettlementStatus};
use crate::utils::app_config::AppConfig;
use anyhow::{anyhow, Result};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;
use contract_integrator::utils::functions::{ContractCallInput, ContractCallOutput};
use contract_integrator::utils::functions::*;

enum OrderActionSide {
    Bid,
    Ask
}

fn can_execute_onchain()->bool {
    env::var("DISABLE_ONCHAIN_INTERACTIONS").unwrap_or("false".to_string()) != "true".to_string()
}

pub async fn unlock_asset(
    config: &mut AppConfig,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet_id: Uuid,
    asset: Uuid,
    amount: u64
) -> Result<()> {

    let execute = can_execute_onchain();

    if !execute {
        return Ok(());
    }

    let wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;
         cradlewalletaccounts.filter(
            id.eq(wallet_id)
        ).get_result::<CradleWalletAccountRecord>(conn)
    }?;

    let asset_record = {
        use crate::schema::asset_book::dsl::*;

        asset_book.filter(
            id.eq(asset)
        ).get_result::<AssetBookRecord>(conn)
    }?;

    let _ = config.wallet.execute(
        contract_integrator::utils::functions::ContractCallInput::CradleAccount(
            contract_integrator::utils::functions::cradle_account::CradleAccountFunctionInput::UnLockAsset(
              contract_integrator::utils::functions::cradle_account::UnLockAssetArgs {
                  asset: asset_record.token,
                  amount,
                  account_contract_id: wallet.contract_id
              }  
            )
        )
    ).await?;
    
   
    Ok(())
}


pub async fn lock_asset(
    config: &mut AppConfig,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    wallet_id: Uuid,
    asset: Uuid,
    amount: u64
)-> Result<()> {

    
    let execute = can_execute_onchain();

    if !execute {
        return Ok(());
    }
    
    let wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;
         cradlewalletaccounts.filter(
            id.eq(wallet_id)
        ).get_result::<CradleWalletAccountRecord>(conn)
    }?;

    let asset_record = {
        use crate::schema::asset_book::dsl::*;

        asset_book.filter(
            id.eq(asset)
        ).get_result::<AssetBookRecord>(conn)
    }?;

    let _ = config.wallet.execute(
        ContractCallInput::CradleAccount(
            cradle_account::CradleAccountFunctionInput::LockAsset(
                cradle_account::LockAssetArgs {
                    asset: asset_record.token,
                    amount,
                    account_contract_id: wallet.contract_id
                }
            )
        )
    ).await?;
    
    Ok(())
}

pub async fn settle_order(
    action_wallet: &mut ActionWallet,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    order_id: Uuid
)-> Result<()> {

    let trades = {
        use crate::schema::orderbooktrades::dsl::*;

        orderbooktrades.filter(
            taker_order_id.eq(order_id).and(
                settlement_status.eq(
                    SettlementStatus::Matched
                )
            )
        ).get_results::<OrderBookTradeRecord>(conn)       
    }?;

    
    for trade in trades {
        let ( maker_order, maker_asset, maker_wallet  ) = get_order_data(conn, trade.maker_order_id)?;          
        let ( taker_order, taker_asset, taker_wallet) = get_order_data(conn, trade.taker_order_id)?;

        // TODO: trigger unlocking within contract itself
        let settlement_tx_id = match settle_onchain(
            action_wallet,
            maker_wallet.clone(),
            taker_wallet.clone(),
            trade.taker_filled_amount.clone(),
            trade.maker_filled_amount.clone(),
            taker_asset,
            maker_asset
        ).await {
            Ok(tx)=>tx,
            Err(e)=>{
                println!("Settlement Failed with error:: {:?}", e);
                // TODO: add more graceful error handling so that the amount that eventually gets unlocked is valid
                continue;
            }
        };


        println!("Settlement tx id :: {:?}", settlement_tx_id);

        record_settled_order(conn, trade.id, settlement_tx_id)?;

        let maker_bid_fill = update_order_fill(
            conn,
            maker_order.id,
            maker_order.bid_asset,
            trade.maker_filled_amount.clone()
        ).await?;

        let maker_ask_fill = update_order_fill(
            conn,
            maker_order.id,
            maker_order.ask_asset,
            trade.taker_filled_amount.clone()
        ).await?;

        let maker_order_status = close_order(
            conn,
            maker_order.id,
            maker_bid_fill,
            maker_ask_fill
        ).await?;

        let taker_bid_fill = update_order_fill(
            conn,
            taker_order.id,
            taker_order.bid_asset,
            trade.taker_filled_amount.clone()
        ).await?;

        let taker_ask_fill = update_order_fill(
            conn,
            taker_order.id,
            taker_order.ask_asset,
            trade.maker_filled_amount.clone()
        ).await?;

        
        let taker_order_status = close_order(
            conn,
            taker_order.id,
            taker_bid_fill,
            taker_ask_fill
        ).await?;
        

        println!("Taker Order Status:: {:?} Maker Order Status {:?}", taker_order_status, maker_order_status);

             

           
        
    }

    

    Ok(())
    
}



pub fn get_order_data(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    orderbook_id: Uuid
) -> Result<(OrderBookRecord, AssetBookRecord, CradleWalletAccountRecord)> {

    let order = {
        use crate::schema::orderbook::dsl::*;

        orderbook.filter(
            id.eq(orderbook_id)
        ).get_result::<OrderBookRecord>(conn)
    }?;

    let asset = {
        use crate::schema::asset_book::dsl::*;

        asset_book.filter(
            id.eq(order.ask_asset)
        ).get_result::<AssetBookRecord>(conn)
    }?;

    let wallet = {
        use crate::schema::cradlewalletaccounts::dsl::*;

        cradlewalletaccounts.filter(
            id.eq(
                order.wallet
            )
        ).get_result::<CradleWalletAccountRecord>(conn)
    }?;

    Ok((order, asset, wallet))
}

pub async fn asset_transfer(
    wallet: &mut ActionWallet,
    sender_account: CradleWalletAccountRecord,
    amount: BigDecimal,
    sending_asset: AssetBookRecord,
    receiver_account: CradleWalletAccountRecord
)-> Result<String> {


    let execute = can_execute_onchain();

    if !execute {
        return Ok(Uuid::new_v4().to_string());
    }
    
    let normalized_amount = amount.to_u64().ok_or_else(|| anyhow!("Amount too large"))?;
    
    let res = wallet.execute(
        ContractCallInput::CradleAccount(
            cradle_account::CradleAccountFunctionInput::TransferAsset(
                TransferAssetArgs {
                    account_contract_id: sender_account.contract_id,
                    asset: sending_asset.token,
                    amount: normalized_amount,
                    to: receiver_account.address
                    
                }
            )
        )
    ).await?;

    match res {
        ContractCallOutput::CradleAccount(cradle_account::CradleAccountFunctionOutput::TransferAsset(output))=>{
            Ok(output.transaction_id)  
        },
        _=>Err(anyhow!("Failed to complete transaction"))
    }
    
}

pub async fn settle_onchain(
    wallet: &mut ActionWallet,
    maker: CradleWalletAccountRecord,
    taker: CradleWalletAccountRecord,
    _maker_transfer_amount: BigDecimal,
    _taker_transfer_amount: BigDecimal,
    maker_transfer_asset: AssetBookRecord,
    taker_transfer_asset: AssetBookRecord
)-> Result<String> {

    
    let execute = can_execute_onchain();

    if !execute {
        return Ok(Uuid::new_v4().to_string());
    }
    let maker_transfer_amount = _maker_transfer_amount.to_u64().ok_or_else(||anyhow!("value too big"))?;
    let taker_transfer_amount = _taker_transfer_amount.to_u64().ok_or_else(||anyhow!("value too big"))?;

    println!("Maker Address:: {:?} ", maker.address.clone());
    println!("Taker Address:: {:?}", taker.address.clone());
    println!("Bid Asset:: {:?}", maker_transfer_asset.token.clone());
    println!("Ask Asset:: {:?} ", taker_transfer_asset.token.clone());
    println!("Bid Amount:: {:?} ", maker_transfer_amount.to_string());
    println!("Ask Amount:: {:?} ", taker_transfer_amount.to_string());
    
    let res = wallet.execute(
       ContractCallInput::OrderBookSettler(
           orderbook_settler::OrderBookSettlerFunctionInput::SettleOrder(
               orderbook_settler::SettleOrderInputArgs {
                   bidder: maker.address,
                   asker: taker.address,
                   bid_asset: taker_transfer_asset.token,
                   ask_asset: maker_transfer_asset.token,
                   bid_asset_amount: taker_transfer_amount,
                   ask_asset_amount: maker_transfer_amount
               }
           )
       )
    ).await?;

    match res {
        ContractCallOutput::OrderBookSettler(OrderBookSettlerFunctionOutput::SettleOrder(output))=>{
            Ok(output.transaction_id)  
        },
        _=>Err(anyhow!("Failed to complete transaction"))
    }
    
}
pub fn record_settled_order(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    trade_id: Uuid,
    settlement_id: String
)-> Result<()> {
    use crate::schema::orderbooktrades::dsl::*;

    let _ = diesel::update(crate::schema::orderbooktrades::table).filter(
        id.eq(trade_id)
    ).set((
            settlement_status.eq(SettlementStatus::Settled),
            settled_at.eq(Utc::now().naive_utc()),
            settlement_tx.eq(settlement_id)
        )).execute(conn)?;

        Ok(())
}



pub async fn update_order_fill(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    order_id: Uuid,
    filled_asset: Uuid,
    filled_amount: BigDecimal
)->Result<(BigDecimal, BigDecimal)> {
    use crate::schema::orderbook::dsl::*;
    use crate::schema::orderbook::table as OrderBookTable;

    let order = orderbook.filter(
        id.eq(order_id)
    ).get_result::<OrderBookRecord>(conn)?;

    let side = if order.bid_asset == filled_asset {
        OrderActionSide::Bid
    }else {
        OrderActionSide::Ask
    };

    let new_filled_amount = match side {
        OrderActionSide::Ask=>{
            order.filled_ask_amount.clone() + filled_amount
        },
        OrderActionSide::Bid=>{
            order.filled_bid_amount.clone() + filled_amount  
        }
    };

    let remaining = match side {
        OrderActionSide::Bid=>{
            (order.bid_amount - new_filled_amount.clone(), order.ask_amount - order.filled_ask_amount)
        },
        OrderActionSide::Ask=>{
            (order.ask_amount - new_filled_amount.clone(), order.bid_amount - order.filled_bid_amount)
        }
    };

    let _ =  match side {
        OrderActionSide::Bid=>{
            diesel::update(OrderBookTable).filter(
                id.eq(order_id)
            ).set(
         filled_bid_amount.eq(new_filled_amount.clone()),
            ).execute(conn)?
         },
        OrderActionSide::Ask=>{
            diesel::update(OrderBookTable).filter(
                id.eq(order_id)
            ).set(
                filled_ask_amount.eq(new_filled_amount),
            ).execute(conn)?
         }
    };
    
    
    Ok(remaining)
}


pub async fn close_order(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    order_id: Uuid,
    bid: (BigDecimal, BigDecimal),
    ask: (BigDecimal, BigDecimal)
)-> Result<OrderStatus> {
    let zero = BigDecimal::from(0);
    use crate::schema::orderbook::dsl::*;
    use crate::schema::orderbook::table as OrderBookTable;

    let (remaining_bid, _) = bid;
    let (remaining_ask, _) = ask;

    let combined = remaining_bid + remaining_ask;

    if combined != zero {
        return Ok(OrderStatus::Open);
    };

    let _ = diesel::update(OrderBookTable)
        .filter(
            id.eq(order_id)
        ).set((
            filled_at.eq(Utc::now().naive_utc()),
            status.eq(OrderStatus::Closed)
        )).execute(conn)?;
    
    
    Ok(OrderStatus::Closed)
}


pub async fn update_order_status(
    config: &mut AppConfig,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    order_id: Uuid,
    order_status: OrderStatus
)-> Result<()> {
    use crate::schema::orderbook::dsl::*;
    use crate::schema::orderbook::table as OrderBookTable;

    let order_data = diesel::update(OrderBookTable)
    .filter(id.eq(order_id))
    .set(
        status.eq(&order_status)
    ).get_result::<OrderBookRecord>(conn)?;

    match order_status {
        OrderStatus::Cancelled=>{
            // then we gotta unlock the assets too
            unlock_asset(
                config,
                conn,
                order_data.wallet,
                order_data.ask_asset,
                order_data.ask_amount.to_u64().ok_or_else(||anyhow!("Unable to unwrap u64"))?
            ).await?;
        },
        _=>{
            // do nothing for close, open won't be used in this case
        }
    }

    Ok(())

}



