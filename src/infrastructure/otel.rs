use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace as sdktrace, Resource};
use std::time::Duration;
use anyhow::{Context, Result};

pub fn init_otel(service_name: &str, endpoint: &str, auth_header: Option<String>) -> Result<sdktrace::Tracer> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let mut otlp_exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(3));

    if let Some(auth) = auth_header {
        otlp_exporter = otlp_exporter.with_headers(vec![(
            "Authorization".parse().unwrap(),
            auth.parse().unwrap(),
        )].into_iter().collect());
    }

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                service_name.to_string(),
            )])),
        )
        .install_batch(runtime::Tokio)
        .context("Failed to initialize OpenTelemetry tracer")?;

    Ok(tracer)
}

pub fn shutdown_otel() {
    global::shutdown_tracer_provider();
}
