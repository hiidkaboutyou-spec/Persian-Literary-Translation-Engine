pub trait Telemetry {
    fn record_error(&self, category: &str);
    fn record_event(&self, name: &str);
}

pub struct NoopTelemetry;

impl Telemetry for NoopTelemetry {
    fn record_error(&self, _category: &str) {}

    fn record_event(&self, _name: &str) {}
}
