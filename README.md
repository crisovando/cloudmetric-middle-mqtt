# middle-mqtt

Gateway middleware que actúa como puente entre un broker MQTT y clientes WebSocket. Recibe telemetría de salud de servidores, la valida y la transmite en tiempo real a dashboards web.

## Objetivo

Conectar un pipeline de datos IoT (MQTT) con un frontend web (React) que necesita visualizar en vivo el estado de salud de servidores simulados. middle-mqtt se suscribe a un tópico MQTT, aplica reglas de validación sobre los datos y los difunde a todos los clientes WebSocket conectados.

## Arquitectura

```
Simulador externo                    Cliente React
       │                                   ▲
       │  publishes MQTT                   │  WebSocket JSON
       ▼                                   │
┌─────────────────┐     ┌───────────────────────────────────┐
│  HiveMQ Broker   │     │           middle-mqtt             │
│ (broker.hivemq   │     │                                   │
│  .com:1883)      │────▶│  rumqttc → broadcast → axum WS   │
└─────────────────┘     │              channel               │
                        └───────────────────────────────────┘
```

### Flujo de datos

1. **Ingesta MQTT** — `rumqttc::AsyncClient` se suscribe al tópico `cloudmetric/simulator/health_data` en un broker público HiveMQ.
2. **Broadcast interno** — Cada mensaje recibido se parsea como `ServerHealth` (JSON) y se envía a un canal `tokio::sync::broadcast` con capacidad para 16 eventos.
3. **Validación** — Antes de enviar a cada cliente WebSocket, se evalúan umbrales de salud (CPU, memoria, temperatura, red). Si algún umbral se excede, el campo `status` se fuerza a `"unhealthy"`.
4. **Difusión WebSocket** — Todos los clientes conectados a `ws://localhost:8080/ws` reciben el evento en tiempo real.

## Estructura del proyecto

| Archivo | Descripción |
|---|---|
| `src/main.rs` | Punto de entrada, configuración del cliente MQTT, servidor HTTP/WS con Axum, CORS |
| `src/device_health.rs` | Modelos de datos: `ServerHealth`, `DeviceHealth`, `NetworkStats` con Serde |
| `src/validation.rs` | Funciones de validación de umbrales: CPU, memoria, temperatura, congestión de red |

## Validación de salud

Un dispositivo se marca como `"unhealthy"` si cumple **alguna** de estas condiciones:

| Métrica | Umbral |
|---|---|
| CPU | > 80.0 % |
| Memoria | > 80.0 % |
| Temperatura | > 75.0 °C |
| Red (in/out) | > 100.0 |

Si no se excede ningún umbral, el `status` se establece como `"healthy"`.

## Formato de datos

```json
{
  "serverId": "servidor-01",
  "health": {
    "timestamp": "2025-01-01T00:00:00Z",
    "cpu": 45.2,
    "temp": 68.1,
    "memory": 72.0,
    "network": {
      "in_value": 56.3,
      "out_value": 42.7
    },
    "uptime": 123456,
    "status": "healthy"
  }
}
```

## Requisitos

- Rust (edición 2024). Instalar desde [rustup.rs](https://rustup.rs/).

### Dependencias principales

| Crate | Versión | Propósito |
|---|---|---|
| `rumqttc` | 0.25.1 | Cliente MQTT asíncrono |
| `serde` / `serde_json` | 1.x | Serialización/deserialización JSON |
| `tokio` | 1 (full) | Runtime asíncrono |
| `axum` | 0.7 (ws) | Servidor web y WebSocket |
| `tower-http` | 0.6 (trace, cors) | Middleware CORS y tracing |

## Inicio rápido

```bash
# Clonar el repositorio
git clone <url-del-repo>
cd middle-mqtt

# Construir
cargo build

# Ejecutar
cargo run
```

El servidor se inicia en `0.0.0.0:8080`.

## Endpoints

| Ruta | Tipo | Descripción |
|---|---|---|
| `ws://localhost:8080/ws` | WebSocket | Recibe eventos JSON de salud de servidores en tiempo real |

## Tecnologías

- **Rust** — Lenguaje de sistema seguro y concurrente
- **Axum** — Framework web ergonómico sobre Tokio, Hyper y Tower
- **rumqttc** — Cliente MQTT asíncrono y liviano
- **Tokio** — Runtime asíncrono con scheduler work-stealing
- **Serde** — Framework de serialización de datos

## Notas de desarrollo

- El proyecto se encuentra en etapa de desarrollo temprano.
- Los valores de configuración (broker MQTT, tópico, puerto, umbrales) están hardcodeados. Se recomienda externalizar a variables de entorno o archivo de configuración para uso productivo.
- La conexión MQTT es sobre TCP plano (puerto 1883, sin TLS).
- `CorsLayer::permissive()` permite todos los orígenes — ajustar para producción.
- No hay manejo de señales (graceful shutdown).
- Sin tests automatizados aún.
