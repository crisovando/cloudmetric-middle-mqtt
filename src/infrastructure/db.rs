use libsql::{Builder, Connection};
use std::env;

pub async fn init_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let db_url = env::var("TURSO_URL").expect("Falta la variable de entorno TURSO_URL");
    let auth_token = env::var("TURSO_TOKEN").expect("Falta la variable de entorno TURSO_TOKEN");

    let db = Builder::new_remote(db_url, auth_token.clone())
        .build()
        .await
        .expect("Error al conectar a la base de datos");

    let conn = db.connect().expect("Error al conectar a la base de datos");

    init_db_with_migrations(&conn).await?;

    println!("Conexión a Turso establecida correctamente.");

    Ok(conn)
}

pub async fn init_db_with_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Ejecutar migraciones
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );
        ",
        (),
    )
    .await?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS alerts (
            id TEXT PRIMARY KEY,
            server_id TEXT NOT NULL,
            server_name TEXT NOT NULL,
            metric TEXT NOT NULL,
            value REAL,
            timestamp INTEGER NOT NULL,
            resolved_at INTEGER
        );
        ",
        (),
    )
    .await?;

    Ok(())
}
