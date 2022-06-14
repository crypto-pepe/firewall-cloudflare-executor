use crate::{errors::ServerError, models::DbConn};

use bb8::Pool;
use bb8_diesel::{DieselConnection, DieselConnectionManager};
use diesel::PgConnection;

pub async fn get_db_pool(conn_string: String) -> Result<Pool<DbConn>, ServerError> {
    let pg_mgr = DieselConnectionManager::<DieselConnection<PgConnection>>::new(conn_string);
    return bb8::Pool::builder()
        .build(pg_mgr)
        .await
        .map_err(|e| ServerError::PoolError(e.to_string()));
}
