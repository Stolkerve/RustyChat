use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

const DB_URL: &str = "sqlite://sqlite.db";

const USER_TABLE: &str = "
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(30) NOT NULL UNIQUE,
    password VARCHAR(300) NOT NULL
  );
";

pub async fn connect_db() -> Pool<Sqlite> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Database created"),
            Err(err) => panic!("Error while creating database: {}", err),
        }
    }

    SqlitePool::connect(DB_URL).await.unwrap()
}

pub async fn create_tables(db: &Pool<Sqlite>) {
    sqlx::query(USER_TABLE).execute(db).await.unwrap();
}
