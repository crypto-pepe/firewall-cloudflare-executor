use opentelemetry::sdk::trace::Tracer;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::Formatter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::EnvFilter;

use crate::configuration::Settings;

pub fn get_subscriber(
    cfg: &Settings,
) -> (
    Box<dyn Subscriber + Send + Sync>,
    Handle<EnvFilter, Formatter>,
) {
    let formatting_layer =
        BunyanFormattingLayer::new(cfg.tracing.svc_name.to_string(), std::io::stdout);
    let sb = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_filter_reloading();
    let h = sb.reload_handle();
    let sb = sb.finish();
    let reg = sb.with(JsonStorageLayer).with(formatting_layer);

    if cfg.tracing.jaeger_endpoint.is_some() {
        let ep = cfg.tracing.jaeger_endpoint.as_ref().unwrap();
        let tracer = init_tracer(cfg.tracing.svc_name.to_string(), ep.into());
        (
            Box::new(reg.with(tracing_opentelemetry::layer().with_tracer(tracer))),
            h,
        )
    } else {
        (Box::new(reg), h)
    }
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

fn init_tracer(svc_name: String, endpoint: String) -> Tracer {
    opentelemetry_jaeger::new_pipeline()
        .with_service_name(svc_name)
        .with_agent_endpoint(endpoint)
        .install_simple()
        .expect("Failed to install tracer")
}
