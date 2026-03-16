# Rust S3 ASR

A high-performance, event-driven Automatic Speech Recognition (ASR) service built in Rust. This service listens for MQTT triggers, downloads video/audio files from S3/MinIO, performs transcription using LiteLLM (Whisper), and publishes the results back to MQTT.

## 🚀 Overview

`rust-s3-asr` is designed as a microservice in a larger Kubernetes-based monitoring and automation ecosystem (Kusanagi). It follows hexagonal architecture principles to ensure maintainability, testability, and clear separation of concerns.

### Key Features
- **Event-Driven**: Triggers processing via MQTT messages.
- **S3 Integration**: Support for S3-compatible storage (e.g., MinIO).
- **LiteLLM / Whisper**: High-quality ASR via OpenAI-compatible endpoints.
- **Observability**: Integrated with OpenTelemetry (OTLP) and structured logging (tracing).
- **Hexagonal Architecture**: Clean separation between Domain logic, Application use cases, and Infrastructure adapters.

## 🏗️ Architecture

The project follows the **Ports and Adapters (Hexagonal)** pattern:

- **Domain**: Core business entities and ports (traits).
- **Application**: Use cases that coordinate the flow of data.
- **Infrastructure**: Concrete implementations of ports (MQTT client, S3 client, LLM client).

```
src/
├── domain/            # Cœur métier (Entities + Ports)
├── application/       # Cas d'usage (ProcessVideoUseCase)
├── infrastructure/    # Adapters (S3, MQTT, LLM, OTEL)
└── main.rs            # Application entry point & configuration
```

## ⚙️ Configuration

The application is configured via environment variables (or a `.env` file):

### MQTT Settings
| Variable | Description | Default |
|----------|-------------|---------|
| `MQTT_HOST` | Hostname of the MQTT broker | `localhost` |
| `MQTT_PORT` | Port of the MQTT broker | `1883` |
| `MQTT_INPUT_TOPIC` | Topic to listen for S3 keys | `input/video` |
| `MQTT_OUTPUT_TOPIC` | Topic to publish transcriptions | `output/transcription` |
| `MQTT_USER` | (Optional) MQTT username | - |
| `MQTT_PASSWORD` | (Optional) MQTT password | - |

### S3 Settings
| Variable | Description | Default |
|----------|-------------|---------|
| `S3_BUCKET` | **(Required)** Target S3 bucket | - |
| `S3_ENDPOINT` | S3 API endpoint | `https://minio-170-api.zacharie.org` |
| `S3_REGION` | S3 region | `us-east-1` |
| `S3_ACCESS_KEY` | **(Required)** S3 access key | - |
| `S3_SECRET_KEY` | **(Required)** S3 secret key | - |
| `S3_IGNORE_SSL` | Disable SSL verification (for dev) | `false` |

### LLM / ASR Settings
| Variable | Description | Default |
|----------|-------------|---------|
| `LLM_URL` | LiteLLM or OpenAI-compatible URL | `http://localhost:4000` |
| `LLM_API_KEY` | (Optional) API key for the LLM | - |
| `LLM_MODEL` | Model to use for ASR | `whisper-1` |

### Observability
| Variable | Description | Default |
|----------|-------------|---------|
| `OTEL_ENABLED` | Enable OpenTelemetry tracing | `false` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP gRPC/HTTP endpoint | `http://o2c-openobserve-collector-gateway-collector.openobserve-collector.svc:4318` |
| `OTEL_EXPORTER_OTLP_HEADERS` | (Optional) Headers for auth | - |

## 🛠️ Usage

### Triggering a Processing Task
Publish an MQTT message to the `MQTT_INPUT_TOPIC` containing the S3 key of the file to process:

```bash
mosquitto_pub -h localhost -t "input/video" -m "recordings/2024-03-16_meeting.mp4"
```

### The Workflow
1. **Trigger**: Application receives the S3 key via MQTT.
2. **Download**: `S3Repository` fetches the file content from the configured bucket.
3. **Transcription**: `LlmRepository` sends the file to the transcription endpoint.
4. **Output**: Transcription results are published to the `MQTT_OUTPUT_TOPIC`.

## 📦 Building

To build the release binary:

```bash
cargo build --release
```

To run with environment variables:

```bash
export S3_BUCKET=my-bucket
export S3_ACCESS_KEY=xxx
export S3_SECRET_KEY=xxx
cargo run
```

---
**Author**: Joseph Zacharie