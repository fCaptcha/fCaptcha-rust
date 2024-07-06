use std::sync::{Arc, Mutex};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::RwLock;
use crate::commons::error::DortCapError::DetailedInternalErr;
use crate::commons::error::FCaptchaResult;

#[derive(Deserialize, Serialize)]
pub struct User {
    pub balance: f64,
    #[serde(skip)]
    pub threads: RwLock<u128>,
    pub api_key: String,
    pub thread_limit: u128,
    pub is_pay_per_use: bool
}

impl User {

    async fn handle_start_req(&self) -> FCaptchaResult<()> {
        if *self.threads.read().await < self.thread_limit {
            *self.threads.write().await += 1;
            return Ok(());
        }
        Err(DetailedInternalErr("Too many threads."))
    }

    async fn handle_end_req(&self) -> FCaptchaResult<()> {
        *self.threads.write().await -= 1;
        Ok(())
    }

    async fn handle_use(&mut self, database: &mut ConnectionManager, price: f64) -> FCaptchaResult<()> {
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
    async fn add_balance(&mut self, database: &mut ConnectionManager, paid_invoice_amount: f64) -> FCaptchaResult<()> {
        if self.balance > 0.0 {
            self.balance += paid_invoice_amount;
            let _result: String = database.hset(&*self.api_key, "balance", self.balance).await?;
            return Ok(());
        }
        Err(DetailedInternalErr("Balance failed to apply."))
    }
}

mod des {}