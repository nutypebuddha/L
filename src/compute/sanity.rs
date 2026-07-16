#[derive(Debug, Clone)]
pub struct CheckResult {
    pub category: String,
    pub val: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub typical: String,
}

#[derive(Debug, Clone)]
pub struct EstimateResult {
    pub val: f64,
    pub order: i32,
    pub context: String,
    pub below_val: f64,
    pub below_name: String,
    pub above_val: f64,
    pub above_name: String,
}

pub fn check(args: &str) -> Option<CheckResult> {
    let args = args.trim().as_bytes();
    let (val_str, rest) = split_token(args)?;
    let rest = trim(rest);
    if !rest.starts_with(b"as ") {
        return None;
    }
    let category = std::str::from_utf8(trim(&rest[3..])).ok()?;
    let val = parse_f64(val_str)?;

    match category {
        "speed_mph" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 300.0,
            unit: "mph".to_string(),
            typical: "highway ~65, city ~30".to_string(),
        }),
        "speed_ms" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 340.0,
            unit: "m/s".to_string(),
            typical: "walking 1.4, running 5, car 27".to_string(),
        }),
        "temp_c" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: -100.0,
            max: 200.0,
            unit: "C".to_string(),
            typical: "room ~20, body ~37, freezing 0".to_string(),
        }),
        "height_m" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 10000.0,
            unit: "m".to_string(),
            typical: "human 1.5-2.0, building 3-300".to_string(),
        }),
        "weight_kg" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 1e6,
            unit: "kg".to_string(),
            typical: "human 50-100, car 1000-2000".to_string(),
        }),
        "energy_j" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 1e15,
            unit: "J".to_string(),
            typical: "food 2000 kcal=8.4e6, bomb 4e9".to_string(),
        }),
        "power_w" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 1e9,
            unit: "W".to_string(),
            typical: "bulb 60, hairdryer 1500, house 1000-5000".to_string(),
        }),
        "distance_km" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 1e6,
            unit: "km".to_string(),
            typical: "walk 5, drive 50, flight 1000".to_string(),
        }),
        "time_s" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 1e8,
            unit: "s".to_string(),
            typical: "blink 0.1, minute 60, hour 3600".to_string(),
        }),
        "price_usd" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 1e9,
            unit: "USD".to_string(),
            typical: "coffee 5, phone 1000, car 30000".to_string(),
        }),
        "percent" => Some(CheckResult {
            category: category.to_string(),
            val,
            min: 0.0,
            max: 100.0,
            unit: "%".to_string(),
            typical: "0-100 range".to_string(),
        }),
        _ => None,
    }
}

pub fn estimate(args: &str) -> Option<EstimateResult> {
    let args = args.trim();
    let val = args.parse::<f64>().ok()?;
    let v = val.abs();
    if v == 0.0 {
        return Some(EstimateResult {
            val,
            order: 0,
            context: "nothing".to_string(),
            below_val: 0.0,
            below_name: String::new(),
            above_val: 0.0,
            above_name: String::new(),
        });
    }
    let exp = v.log10().floor() as i32;
    let context = match exp {
        -3..=-1 => "grain of sand",
        0 => "pebble",
        1 => "apple",
        2 => "meal",
        3 => "human",
        4 => "car",
        5 => "house",
        6 => "building",
        7 => "skyscraper",
        8 => "city",
        9 => "country",
        10 => "planet",
        11 => "star",
        12 => "solar system",
        13..=15 => "galaxy",
        16..=20 => "universe",
        _ => "beyond universe",
    };

    let references: &[(f64, &str)] = &[
        (1e-30, "proton mass (kg)"),
        (1e-27, "hydrogen atom (kg)"),
        (1e-23, "Avogadro-scale (1/mol)"),
        (1e-15, "femtometer (fm)"),
        (1e-12, "picometer (pm)"),
        (1e-9, "nanometer (nm) -- virus scale"),
        (1e-6, "micrometer (um) -- cell scale"),
        (1e-3, "millimeter (mm)"),
        (1e-2, "centimeter (cm)"),
        (1.0, "meter/second/kilogram (SI base)"),
        (1e1, "tens -- human-scale"),
        (1e2, "hundreds"),
        (1e3, "thousands -- kilo"),
        (1e4, "10,000"),
        (1e5, "100,000"),
        (1e6, "million -- mega"),
        (1e9, "billion -- giga"),
        (1e12, "trillion -- tera"),
        (1e15, "quadrillion"),
        (1e23, "Avogadro's number"),
        (1e30, "sun mass (kg)"),
    ];

    let mut below = (0.0f64, "");
    let mut above = (0.0f64, "");
    for &(ref_val, ref_name) in references {
        if ref_val <= v {
            below = (ref_val, ref_name);
        } else if above.0 == 0.0 {
            above = (ref_val, ref_name);
        }
    }

    Some(EstimateResult {
        val,
        order: exp,
        context: context.to_string(),
        below_val: below.0,
        below_name: below.1.to_string(),
        above_val: above.0,
        above_name: above.1.to_string(),
    })
}

pub fn list_categories() -> Vec<(&'static str, &'static str)> {
    vec![
        ("speed_mph", "0-300 mph"),
        ("speed_ms", "0-340 m/s"),
        ("temp_c", "-100 to 200 C"),
        ("height_m", "0-10000 m"),
        ("weight_kg", "0-1e6 kg"),
        ("energy_j", "0-1e15 J"),
        ("power_w", "0-1e9 W"),
        ("distance_km", "0-1e6 km"),
        ("time_s", "0-1e8 s"),
        ("price_usd", "0-1e9 USD"),
        ("percent", "0-100%"),
    ]
}

fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len() && (s[start] == b' ' || s[start] == b'\t') {
        start += 1;
    }
    let mut end = s.len();
    while end > start && (s[end - 1] == b' ' || s[end - 1] == b'\t') {
        end -= 1;
    }
    &s[start..end]
}

fn split_token(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let s = trim(s);
    if s.is_empty() {
        return None;
    }
    let mut i = 0;
    while i < s.len() && s[i] != b' ' && s[i] != b'\t' {
        i += 1;
    }
    if i >= s.len() {
        return None;
    }
    Some((&s[..i], &s[i..]))
}

fn parse_f64(s: &[u8]) -> Option<f64> {
    let s = trim(s);
    let s_str = std::str::from_utf8(s).ok()?;
    s_str.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_speed() {
        let result = check("60 as speed_mph").unwrap();
        assert_eq!(result.category, "speed_mph");
        assert_eq!(result.val, 60.0);
    }

    #[test]
    fn test_check_invalid() {
        assert!(check("100 as invalid_category").is_none());
    }

    #[test]
    fn test_estimate() {
        let result = estimate("1000").unwrap();
        assert_eq!(result.order, 3);
        assert_eq!(result.context, "human");
    }

    #[test]
    fn test_estimate_zero() {
        let result = estimate("0").unwrap();
        assert_eq!(result.context, "nothing");
    }

    #[test]
    fn test_list_categories() {
        let categories = list_categories();
        assert!(!categories.is_empty());
    }
}
