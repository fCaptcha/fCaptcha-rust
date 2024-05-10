use async_once::AsyncOnce;
use lazy_static::lazy_static;
use redis::aio::ConnectionManager;
use crate::get_redis_instance;

lazy_static! {
    static ref VALID_ANSWERS: AsyncOnce<ConnectionManager> = AsyncOnce::new(async {
        return get_redis_instance(314).await;
    });
}