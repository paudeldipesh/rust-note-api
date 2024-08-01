use actix::{Actor, Addr, SyncContext};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

type PgConnectionManagerType = ConnectionManager<PgConnection>;

pub struct AppState {
    pub db: Addr<DbActor>,
}

pub struct DbActor(pub Pool<PgConnectionManagerType>);

impl Actor for DbActor {
    type Context = SyncContext<Self>;
}

pub fn get_pool(db_url: &str) -> Pool<PgConnectionManagerType> {
    let manager: PgConnectionManagerType = ConnectionManager::<PgConnection>::new(db_url);

    let pool: Pool<PgConnectionManagerType> = Pool::builder()
        .build(manager)
        .expect("Error building a connection pool");

    pool
}
