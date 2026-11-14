use lm_sensors::{feature, value, LMSensors};

#[derive(Default)]
pub struct Voltage {
    sensors: Option<LMSensors>,
    pub voltage_read_points: Vec<(String, f64)>,
}

impl Voltage {
    pub fn new(sensors: LMSensors) -> Self {
        Self {
            sensors: Some(sensors),
            voltage_read_points: Vec::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.voltage_read_points.clear();
        let Some(sensors) = &self.sensors else {
            return;
        };

        for chip in sensors.chip_iter(None) {
            for feature in chip.feature_iter() {
                if feature.kind() != Some(feature::Kind::Voltage) {
                    continue;
                }

                let Ok(input) = feature.sub_feature_by_kind(value::Kind::VoltageInput) else {
                    continue;
                };
                let Ok(value) = input.raw_value() else {
                    continue;
                };
                if !value.is_finite() {
                    continue;
                }

                let label = feature.label().unwrap_or_else(|_| {
                    feature
                        .name()
                        .and_then(Result::ok)
                        .unwrap_or("N/A")
                        .to_string()
                });
                self.voltage_read_points.push((label, value));
            }
        }
    }
}
