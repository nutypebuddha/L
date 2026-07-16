pub fn compute_named(op: &str, args: &[f64]) -> Option<f64> {
    match op {
        "add" => {
            if args.len() >= 2 {
                Some(args[0] + args[1])
            } else {
                None
            }
        }
        "sub" => {
            if args.len() >= 2 {
                Some(args[0] - args[1])
            } else {
                None
            }
        }
        "mul" => {
            if args.len() >= 2 {
                Some(args[0] * args[1])
            } else {
                None
            }
        }
        "div" => {
            if args.len() >= 2 && args[1] != 0.0 {
                Some(args[0] / args[1])
            } else {
                None
            }
        }
        "neg" => args.first().map(|x| -x),
        "abs" => args.first().map(|x| x.abs()),
        "hypot" => {
            if args.len() >= 2 {
                Some(args[0].hypot(args[1]))
            } else {
                None
            }
        }
        "sqrt" => args.first().map(|x| x.sqrt()),
        "pct" => {
            if args.len() >= 2 {
                Some(args[0] * args[1] / 100.0)
            } else {
                None
            }
        }
        "pct_change" => {
            if args.len() >= 2 && args[0] != 0.0 {
                Some((args[1] - args[0]) / args[0] * 100.0)
            } else {
                None
            }
        }
        "ratio" => {
            if args.len() >= 3 && args[1] != 0.0 {
                Some(args[0] / args[1] * args[2])
            } else {
                None
            }
        }
        "dilute" => {
            if args.len() >= 2 && args[1] != 0.0 {
                Some(args[0] / args[1])
            } else {
                None
            }
        }
        "energy" => args.first().map(|m| m * 89875517873681764.0),
        "f_to_c" => args.first().map(|f| (f - 32.0) * 5.0 / 9.0),
        "c_to_f" => args.first().map(|c| c * 9.0 / 5.0 + 32.0),
        "c_to_k" => args.first().map(|c| c + 273.15),
        "f_to_k" => args.first().map(|f| (f - 32.0) * 5.0 / 9.0 + 273.15),
        "k_to_c" => args.first().map(|k| k - 273.15),
        "k_to_f" => args.first().map(|k| (k - 273.15) * 9.0 / 5.0 + 32.0),
        "mi_to_km" => args.first().map(|mi| mi * 1.609344),
        "km_to_mi" => args.first().map(|km| km / 1.609344),
        "mph_to_kmh" => args.first().map(|mph| mph * 1.609344),
        "kmh_to_mph" => args.first().map(|kmh| kmh / 1.609344),
        "lb_to_kg" => args.first().map(|lb| lb * 0.453592),
        "kg_to_lb" => args.first().map(|kg| kg / 0.453592),
        "ft_to_m" => args.first().map(|ft| ft * 0.3048),
        "in_to_cm" => args.first().map(|inches| inches * 2.54),
        "mph_to_ms" => args.first().map(|mph| mph * 0.44704),
        "ms_to_mph" => args.first().map(|ms| ms / 0.44704),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_named_add() {
        assert_eq!(compute_named("add", &[2.0, 3.0]), Some(5.0));
    }

    #[test]
    fn test_compute_named_sub() {
        assert_eq!(compute_named("sub", &[10.0, 4.0]), Some(6.0));
    }

    #[test]
    fn test_compute_named_mul() {
        assert_eq!(compute_named("mul", &[3.0, 4.0]), Some(12.0));
    }

    #[test]
    fn test_compute_named_div() {
        assert_eq!(compute_named("div", &[12.0, 4.0]), Some(3.0));
        assert_eq!(compute_named("div", &[12.0, 0.0]), None);
    }

    #[test]
    fn test_compute_named_neg() {
        assert_eq!(compute_named("neg", &[5.0]), Some(-5.0));
    }

    #[test]
    fn test_compute_named_pct() {
        assert_eq!(compute_named("pct", &[200.0, 15.0]), Some(30.0));
    }

    #[test]
    fn test_compute_named_energy() {
        let e = compute_named("energy", &[1.0]).unwrap();
        assert!((e - 89875517873681764.0).abs() < 1e6);
    }
}
