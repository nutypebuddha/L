//! Runnable vertical slice: the Cyberpunk-RED/2077-as-real scenario.
//!
//! Run with: `cargo run -p laverna --example adaptive_cyberpunk`
//!
//! Prints the full `StrategyCycleReport` as machine-readable JSON.

use laverna::adaptive::{run_cyberpunk_scenario, Evidence, Fact, Factor, WorldEvent};

fn main() {
    let event = WorldEvent {
        evidence: Evidence {
            id: "E1".to_string(),
            provenance: "HYPOTHETICAL SCENARIO EVENT".to_string(),
            content:
                "A cross-corporate coalition ratifies the Autonomous-Weapons Treaty and stands up an \
                 independent surveillance audit authority."
                    .to_string(),
            confidence: 0.6,
        },
        factor_adjustments: vec![
            (
                Factor::AutonomousWeapons,
                -0.1,
                "Treaty ratified.".to_string(),
            ),
            (
                Factor::UbiquitousSurveillance,
                -0.05,
                "Audit authority formed.".to_string(),
            ),
            (
                Factor::Cybercrime,
                -0.05,
                "Coordinated enforcement begun.".to_string(),
            ),
        ],
        new_facts: vec![Fact::new(
            "F-new",
            "surveillance-oversight",
            true,
            "Independent surveillance oversight now exists.",
            0.6,
            "HYPOTHETICAL",
        )],
        new_assumptions: vec![
            "Treaty signatories include the three largest autonomous-systems firms.".to_string(),
        ],
    };

    let report = run_cyberpunk_scenario(&event);
    println!("{}", report.to_json());
}
