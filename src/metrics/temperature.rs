use crate::models::{TemperatureMetrics, TemperatureSensor};
use sysinfo::Components;

pub fn collect_temperature_metrics() -> TemperatureMetrics {
    let components = Components::new_with_refreshed_list();
    let sensors: Vec<TemperatureSensor> = components
        .iter()
        .filter(|c| {
            c.temperature()
                .map(|t| t > 0.0 && t < 150.0)
                .unwrap_or(false)
        })
        .map(|c| TemperatureSensor {
            label: c.label().to_string(),
            temperature_celsius: c.temperature(),
            max_celsius: c.max(),
            critical_celsius: c.critical(),
        })
        .collect();

    TemperatureMetrics { sensors }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_temperature_returns_valid_struct() {
        let metrics = collect_temperature_metrics();
        // Sensors may be empty on CI or return unusual values on some hardware
        for sensor in &metrics.sensors {
            assert!(!sensor.label.is_empty());
        }
    }

    #[test]
    fn test_temperature_metrics_serializes() {
        let metrics = collect_temperature_metrics();
        let json = serde_json::to_string(&metrics).expect("serialize");
        assert!(json.contains("sensors"));
    }
}
