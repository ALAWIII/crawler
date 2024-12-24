use std::env;
use std::sync::Arc;
use surrealdb::{
    engine::local::{Db, RocksDb},
    Surreal,
};
use tokio::sync::OnceCell;

static DB_CONNECTION: OnceCell<Arc<Surreal<Db>>> = OnceCell::const_new();

static QUERIES : [&str;8] = [
//"DEFINE NAMESPACE IF NOT EXISTS search;",
//"DEFINE DATABASE IF NOT EXISTS crawl_base;",
"DEFINE TABLE OVERWRITE inv_index TYPE ANY SCHEMAFULL PERMISSIONS NONE;",
"DEFINE TABLE OVERWRITE document TYPE ANY SCHEMAFULL PERMISSIONS NONE;",
"DEFINE FIELD OVERWRITE id ON TABLE document TYPE object ASSERT id.url.is_string();",
"DEFINE FIELD OVERWRITE id ON TABLE inv_index TYPE object
ASSERT id.term.is_string() AND id.doc_url.is_string();",

"DEFINE FIELD OVERWRITE location ON TABLE inv_index TYPE array<int>;",

"DEFINE FIELD OVERWRITE frequency ON TABLE inv_index TYPE int DEFAULT location.len();",
"DEFINE FIELD OVERWRITE doc_length ON TABLE inv_index TYPE int ;",

"DEFINE FIELD OVERWRITE tf ON TABLE inv_index TYPE float DEFAULT <float>frequency/<float>doc_length;",
];

///also creates if its not exist by default and set up the tables and schema
async fn connect_database() -> surrealdb::Result<Arc<Surreal<Db>>> {
    let path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("crawl_base");
    let path_string = path.to_str().unwrap();
    // println!("{}", path_string);
    let db = Surreal::new::<RocksDb>(path_string).await?;

    db.use_ns("search").use_db("crawl_base").await?;

    create_schema(&db).await;

    Ok(Arc::new(db))
}

async fn create_schema(connection: &Surreal<Db>) {
    for query in QUERIES {
        match connection.query(query).await {
            Ok(_) => println!("Successfully executed query: {}", query),
            Err(e) => eprintln!("Error executing query: {} -> {}", query, e),
        };
    }
}
/// initializes the database connection or returning it if it was already established before.
pub async fn get_db_connection() -> Arc<Surreal<Db>> {
    DB_CONNECTION
        .get_or_init(|| async { connect_database().await.unwrap() })
        .await
        .clone()
}
