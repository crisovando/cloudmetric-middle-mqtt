use crate::domain::{state::ServerState, status::HealthStatus};
use libsql::Connection;

pub async fn load_all_servers(
    db: &Connection,
) -> Result<Vec<ServerState>, Box<dyn std::error::Error>> {
    let mut rows = db.query("SELECT id, name FROM servers", ()).await?;
    let mut servers = Vec::new();

    while let Ok(Some(row)) = rows.next().await {
        let db_id: i64 = row.get(0)?;
        let server_id: String = db_id.to_string();

        let state = ServerState {
            server_id,
            name: row.get(1)?,
            status: HealthStatus::Offline,
            ..Default::default()
        };

        servers.push(state);
    }

    Ok(servers)
}

pub async fn add_server(db: &Connection, name: &str) -> Result<i64, Box<dyn std::error::Error>> {
    db.execute(
        "INSERT INTO servers (name) VALUES (?1)",
        libsql::params![name.to_string()],
    )
    .await?;

    let new_id = db.last_insert_rowid();
    Ok(new_id)
}

pub async fn delete_server(
    db: &Connection,
    server_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_id = server_id
        .parse::<i64>()
        .map_err(|_| "El server_id debe ser un número válido")?;

    db.execute(
        "DELETE FROM servers WHERE id = ?1",
        libsql::params![db_id],
    )
    .await?;

    Ok(())
}
