use std::env;

use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::{fmt, EnvFilter, Registry};

pub(crate) fn initialize_tracing_subscriber() {
    // Conditionally add the OpenTelemetry layer if it is initialized
    if let Some(otel_layer) = construct_open_telemetry_layer() {
        let subscriber = Registry::default()
            .with(EnvFilter::from_default_env())
            .with(otel_layer)
            .with(fmt::layer().pretty());

        tracing::subscriber::set_global_default(subscriber)
            .expect("Could not set up global logger");
    } else {
        let subscriber = Registry::default()
            .with(EnvFilter::from_default_env())
            .with(fmt::layer().pretty());

        tracing::subscriber::set_global_default(subscriber)
            .expect("Could not set up global logger");
    };
}

fn construct_open_telemetry_layer(
) -> Option<OpenTelemetryLayer<Layered<EnvFilter, Registry>, sdktrace::Tracer>> {
    // Read the OTEL_TRACES_ENDPOINT environment variable
    let otel_endpoint = env::var("OTEL_TRACES_ENDPOINT").ok()?;

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic() // Use gRPC with Tonic as the transport protocol
        .with_endpoint(otel_endpoint);

    // Build the tracer provider
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            sdktrace::Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "Gamayun",
            )])),
        )
        .install_batch(Tokio)
        .expect("Failed to install OpenTelemetry tracer.");

    // Obtain a tracer from the provider
    let tracer = tracer_provider.tracer("Gamayun");

    Some(OpenTelemetryLayer::new(tracer))
}
