use crate::errors::ServerError;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbConn = ConnectionManager<PgConnection>;

pub async fn get_db_pool(conn_string: String) -> Result<Pool<DbConn>, ServerError> {
    let pg_mgr = ConnectionManager::new(conn_string);
    Pool::builder()
        .build(pg_mgr)
        .map_err(|e| ServerError::PoolError(e.to_string()))
}
