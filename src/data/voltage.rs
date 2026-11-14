use lm_sensors::LMSensors;

pub struct Voltage {
    sensors: Option<LMSensors>,
    pub voltage_read_points: Vec<(String, f64)>
}

impl Voltage {
    pub fn new(sensors: LMSensors) -> Self {
        let voltage_read_points = Vec::new();

        Self {
            sensors: Some(sensors),
            voltage_read_points
        }
    }

    pub fn refresh(&mut self) {
        self.voltage_read_points.clear();
        let Some(sensors) = &self.sensors else { return; };
        for chip in sensors.chip_iter(None) {
            for feature in chip.feature_iter() {
                let name = feature.name().and_then(Result::ok).unwrap_or("N/A");

                if name.contains("in") {
                    for sub_value in feature.sub_feature_iter() {
                        if let Ok(value) = sub_value.raw_value() {
                            let voltage_point = (feature.clone().to_string(), value);
                            self.voltage_read_points.push(voltage_point);
                        }
                    }
                }

            }
        }
    }
}
