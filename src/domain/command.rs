use crate::domain::metric::MetricName;

pub enum ServerCommand {
    SetFailureProbability {
        server_id: String,
        probability: f64,
    },
    SetMetric {
        server_id: String,
        metric: MetricName,
        value: f64,
    },
    ReleaseMetric {
        server_id: String,
        metric: MetricName,
    },
    CreateServer {
        server_id: String,
        name: String,
    },
    DeleteServer {
        server_id: String,
    },
}
