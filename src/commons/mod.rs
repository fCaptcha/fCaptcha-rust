use lazy_static::lazy_static;
use tokio::runtime::{Builder, Runtime};
use crate::ARGUMENTS;

pub(crate) mod utils;
pub mod error;
pub mod console;


lazy_static! {
    pub static ref RUNTIME: Runtime = Builder::new_multi_thread()
        .thread_name("DCWorker")
        .worker_threads(ARGUMENTS.max_sync_tasks)
        .max_blocking_threads(ARGUMENTS.max_sync_tasks)
        .enable_all()
        .build()
        .expect("Runtime failed to initialize");
    pub static ref REDIS_RUNTIME: Runtime = Builder::new_multi_thread()
        .thread_name("DCRedisWorker")
        .worker_threads(70)
        .enable_all()
        .build()
        .expect("Runtime failed to initialize");
}
