use sqlx::PgPool;
use tokio::sync::Notify;

pub struct AppState {
    pub notify: Notify,
    pub pool: PgPool,
}
