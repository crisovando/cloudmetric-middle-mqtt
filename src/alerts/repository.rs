use crate::domain::alert::Alert;
use libsql::Connection;

pub async fn insert_alert(db: &Connection, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
    db.execute(
        "INSERT INTO alerts (id, server_id, server_name, metric, value, timestamp, resolved_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        libsql::params![
            alert.id.to_string(),
            alert.server_id.clone(),
            alert.server_name.clone(),
            serde_json::to_string(&alert.metric)?,
            alert.value,
            alert.timestamp.timestamp(),
            alert.resolved_at.map(|t| t.timestamp()),
        ],
    )
    .await?;

    Ok(())
}

pub async fn load_recent_alerts(
    db: &Connection,
    limit: usize,
) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
    let mut rows = db
        .query(
            "SELECT id, server_id, server_name, metric, value, timestamp, resolved_at
             FROM alerts
             ORDER BY timestamp DESC
             LIMIT ?1",
            libsql::params![limit as i64],
        )
        .await?;

    let mut alerts = Vec::new();

    while let Ok(Some(row)) = rows.next().await {
        let id_str: String = row.get(0)?;
        let id = uuid::Uuid::parse_str(&id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
        let server_id: String = row.get(1)?;
        let server_name: String = row.get(2)?;
        let metric_json: String = row.get(3)?;
        let metric = serde_json::from_str(&metric_json).unwrap_or(crate::domain::alert::AlertMetric::Offline);
        let value: Option<f64> = row.get(4)?;
        let value = value.map(|v| v as f32);
        let ts: i64 = row.get(5)?;
        let timestamp = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default();
        let resolved_at_ts: Option<i64> = row.get(6)?;
        let resolved_at = resolved_at_ts.and_then(|ts| chrono::DateTime::from_timestamp(ts, 0));

        alerts.push(Alert {
            id,
            server_id,
            server_name,
            timestamp,
            metric,
            value,
            resolved_at,
        });
    }

    Ok(alerts)
}

pub async fn resolve_alerts_by_server(
    db: &Connection,
    server_id: &str,
    resolved_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    db.execute(
        "UPDATE alerts SET resolved_at = ?1 WHERE server_id = ?2 AND resolved_at IS NULL",
        libsql::params![resolved_at.timestamp(), server_id],
    )
    .await?;

    Ok(())
}
