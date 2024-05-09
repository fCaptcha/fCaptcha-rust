use std::sync::{Arc, Mutex};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::RwLock;
use crate::commons::error::DortCapError::DetailedInternalErr;
use crate::commons::error::DortCapResult;

#[derive(Deserialize, Serialize)]
pub struct User {
    pub balance: f64,
    #[serde(skip)]
    pub threads: RwLock<u32>,
    pub api_key: String,
    pub thread_limit: u32,
    pub is_pay_per_use: bool
}

impl User {

    async fn handle_start_req(&self) -> DortCapResult<()> {
        if *self.threads.read().await < self.thread_limit {
            *self.threads.write().await += 1;
            return Ok(());
        }
        Err(DetailedInternalErr("Too many threads."))
    }

    async fn handle_end_req(&self) -> DortCapResult<()> {
        *self.threads.write().await -= 1;
        Ok(())
    }

    async fn handle_use(&mut self, database: &mut ConnectionManager, price: f64) -> DortCapResult<()> {
        if self.balance > 0.0 {
            self.balance = f64::max(self.balance - price, 0.0);
            let _result: String = database.hset(&*self.api_key, "balance", self.balance).await?;
            return Ok(());
        }
        Err(DetailedInternalErr("User's balance is too low."))
    }

    /**
    * Adds balance to a user.
    */
    async fn add_balance(&mut self, database: &mut ConnectionManager, paid_invoice_amount: f64) -> DortCapResult<()> {
        if self.balance > 0.0 {
            self.balance += paid_invoice_amount;
            let _result: String = database.hset(&*self.api_key, "balance", self.balance).await?;
            return Ok(());
        }
        Err(DetailedInternalErr("Balance failed to apply."))
    }
}

mod des {}