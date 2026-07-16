pub fn convert_temperature(value: f64, from: &str, to: &str) -> Option<f64> {
    let celsius = match from {
        "c" | "celsius" => value,
        "f" | "fahrenheit" => (value - 32.0) * 5.0 / 9.0,
        "k" | "kelvin" => value - 273.15,
        _ => return None,
    };
    match to {
        "c" | "celsius" => Some(celsius),
        "f" | "fahrenheit" => Some(celsius * 9.0 / 5.0 + 32.0),
        "k" | "kelvin" => Some(celsius + 273.15),
        _ => None,
    }
}

pub fn convert_distance(value: f64, from: &str, to: &str) -> Option<f64> {
    let meters = match from {
        "m" | "meter" | "meters" => value,
        "km" | "kilometer" | "kilometers" => value * 1000.0,
        "cm" | "centimeter" | "centimeters" => value / 100.0,
        "mm" | "millimeter" | "millimeters" => value / 1000.0,
        "mi" | "mile" | "miles" => value * 1609.344,
        "ft" | "foot" | "feet" => value * 0.3048,
        "in" | "inch" | "inches" => value * 0.0254,
        "yd" | "yard" | "yards" => value * 0.9144,
        _ => return None,
    };
    match to {
        "m" | "meter" | "meters" => Some(meters),
        "km" | "kilometer" | "kilometers" => Some(meters / 1000.0),
        "cm" | "centimeter" | "centimeters" => Some(meters * 100.0),
        "mm" | "millimeter" | "millimeters" => Some(meters * 1000.0),
        "mi" | "mile" | "miles" => Some(meters / 1609.344),
        "ft" | "foot" | "feet" => Some(meters / 0.3048),
        "in" | "inch" | "inches" => Some(meters / 0.0254),
        "yd" | "yard" | "yards" => Some(meters / 0.9144),
        _ => None,
    }
}

pub fn convert_weight(value: f64, from: &str, to: &str) -> Option<f64> {
    let kg = match from {
        "kg" | "kilogram" | "kilograms" => value,
        "g" | "gram" | "grams" => value / 1000.0,
        "mg" | "milligram" | "milligrams" => value / 1_000_000.0,
        "lb" | "pound" | "pounds" => value * 0.453592,
        "oz" | "ounce" | "ounces" => value * 0.0283495,
        "t" | "ton" | "tons" => value * 907.185,
        _ => return None,
    };
    match to {
        "kg" | "kilogram" | "kilograms" => Some(kg),
        "g" | "gram" | "grams" => Some(kg * 1000.0),
        "mg" | "milligram" | "milligrams" => Some(kg * 1_000_000.0),
        "lb" | "pound" | "pounds" => Some(kg / 0.453592),
        "oz" | "ounce" | "ounces" => Some(kg / 0.0283495),
        "t" | "ton" | "tons" => Some(kg / 907.185),
        _ => None,
    }
}

pub fn convert_speed(value: f64, from: &str, to: &str) -> Option<f64> {
    let ms = match from {
        "ms" | "m/s" => value,
        "kmh" | "km/h" => value / 3.6,
        "mph" | "mi/h" => value * 0.44704,
        "kn" | "knot" | "knots" => value * 0.514444,
        _ => return None,
    };
    match to {
        "ms" | "m/s" => Some(ms),
        "kmh" | "km/h" => Some(ms * 3.6),
        "mph" | "mi/h" => Some(ms / 0.44704),
        "kn" | "knot" | "knots" => Some(ms / 0.514444),
        _ => None,
    }
}

pub fn convert_time(value: f64, from: &str, to: &str) -> Option<f64> {
    let seconds = match from {
        "s" | "sec" | "second" | "seconds" => value,
        "min" | "minute" | "minutes" => value * 60.0,
        "h" | "hr" | "hour" | "hours" => value * 3600.0,
        "d" | "day" | "days" => value * 86400.0,
        "w" | "week" | "weeks" => value * 604800.0,
        _ => return None,
    };
    match to {
        "s" | "sec" | "second" | "seconds" => Some(seconds),
        "min" | "minute" | "minutes" => Some(seconds / 60.0),
        "h" | "hr" | "hour" | "hours" => Some(seconds / 3600.0),
        "d" | "day" | "days" => Some(seconds / 86400.0),
        "w" | "week" | "weeks" => Some(seconds / 604800.0),
        _ => None,
    }
}

pub fn convert_pressure(value: f64, from: &str, to: &str) -> Option<f64> {
    let pa = match from {
        "pa" | "pascal" => value,
        "kpa" | "kilopascal" => value * 1000.0,
        "atm" => value * 101325.0,
        "psi" => value * 6894.757,
        "bar" => value * 100000.0,
        "mmhg" | "torr" => value * 133.322,
        _ => return None,
    };
    match to {
        "pa" | "pascal" => Some(pa),
        "kpa" | "kilopascal" => Some(pa / 1000.0),
        "atm" => Some(pa / 101325.0),
        "psi" => Some(pa / 6894.757),
        "bar" => Some(pa / 100000.0),
        "mmhg" | "torr" => Some(pa / 133.322),
        _ => None,
    }
}

pub fn convert_energy(value: f64, from: &str, to: &str) -> Option<f64> {
    let joules = match from {
        "j" | "joule" | "joules" => value,
        "cal" | "calorie" | "calories" => value * 4.184,
        "kcal" => value * 4184.0,
        "wh" => value * 3600.0,
        "kwh" => value * 3_600_000.0,
        "ev" => value * 1.602e-19,
        "btu" => value * 1055.06,
        _ => return None,
    };
    match to {
        "j" | "joule" | "joules" => Some(joules),
        "cal" | "calorie" | "calories" => Some(joules / 4.184),
        "kcal" => Some(joules / 4184.0),
        "wh" => Some(joules / 3600.0),
        "kwh" => Some(joules / 3_600_000.0),
        "ev" => Some(joules / 1.602e-19),
        "btu" => Some(joules / 1055.06),
        _ => None,
    }
}

pub fn convert_data(value: f64, from: &str, to: &str) -> Option<f64> {
    let bytes = match from {
        "b" | "byte" | "bytes" => value,
        "kb" => value * 1024.0,
        "mb" => value * 1_048_576.0,
        "gb" => value * 1_073_741_824.0,
        "tb" => value * 1_099_511_627_776.0,
        _ => return None,
    };
    match to {
        "b" | "byte" | "bytes" => Some(bytes),
        "kb" => Some(bytes / 1024.0),
        "mb" => Some(bytes / 1_048_576.0),
        "gb" => Some(bytes / 1_073_741_824.0),
        "tb" => Some(bytes / 1_099_511_627_776.0),
        _ => None,
    }
}

pub fn parse_and_convert(input: &str) -> Option<(f64, String, String)> {
    let input = input.trim();
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let value = parts[0].parse::<f64>().ok()?;
    let from = parts[1].to_lowercase();
    let to = parts[2].to_lowercase();

    let result = match from.as_str() {
        "c" | "f" | "k" | "celsius" | "fahrenheit" | "kelvin" => {
            convert_temperature(value, &from, &to)?
        }
        "m" | "km" | "cm" | "mm" | "mi" | "ft" | "in" | "yd" => {
            convert_distance(value, &from, &to)?
        }
        "kg" | "g" | "mg" | "lb" | "oz" | "t" => convert_weight(value, &from, &to)?,
        "ms" | "m/s" | "kmh" | "km/h" | "mph" | "kn" => convert_speed(value, &from, &to)?,
        "s" | "min" | "h" | "d" | "w" | "sec" | "hr" | "day" | "week" => {
            convert_time(value, &from, &to)?
        }
        "pa" | "kpa" | "atm" | "psi" | "bar" | "mmhg" | "torr" => {
            convert_pressure(value, &from, &to)?
        }
        "j" | "cal" | "kcal" | "wh" | "kwh" | "ev" | "btu" => convert_energy(value, &from, &to)?,
        "b" | "kb" | "mb" | "gb" | "tb" | "byte" | "bytes" => convert_data(value, &from, &to)?,
        _ => return None,
    };

    Some((result, from, to))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_temperature() {
        assert!((convert_temperature(212.0, "f", "c").unwrap() - 100.0).abs() < 1e-10);
        assert!((convert_temperature(0.0, "c", "f").unwrap() - 32.0).abs() < 1e-10);
        assert!((convert_temperature(0.0, "c", "k").unwrap() - 273.15).abs() < 1e-10);
    }

    #[test]
    fn test_convert_distance() {
        assert!((convert_distance(1.0, "mi", "km").unwrap() - 1.609344).abs() < 1e-6);
        assert!((convert_distance(12.0, "in", "cm").unwrap() - 30.48).abs() < 1e-6);
    }

    #[test]
    fn test_convert_weight() {
        assert!((convert_weight(100.0, "lb", "kg").unwrap() - 45.3592).abs() < 1e-4);
    }

    #[test]
    fn test_convert_speed() {
        assert!((convert_speed(60.0, "mph", "m/s").unwrap() - 26.8224).abs() < 1e-4);
    }

    #[test]
    fn test_convert_time() {
        assert!((convert_time(1.0, "h", "min").unwrap() - 60.0).abs() < 1e-10);
        assert!((convert_time(1.0, "d", "h").unwrap() - 24.0).abs() < 1e-10);
    }
}
