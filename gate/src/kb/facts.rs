use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Knowledge domains using Greek letter naming system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// Mathematics & Logic (pure math, proofs, algorithms)
    Alpha,
    /// Physics & Chemistry (physical laws, elements, reactions)
    Beta,
    /// Astronomy & Cosmology (stars, planets, universe)
    Gamma,
    /// Earth & Environment (geography, climate, geology)
    Delta,
    /// Biology & Medicine (life sciences, health, anatomy)
    Epsilon,
    /// Computer Science & AI (computing, ML, algorithms)
    Zeta,
    /// Engineering & Tech (applied science, inventions)
    Eta,
    /// Economics & Finance (markets, money, trade)
    Theta,
    /// History & Anthropology (civilizations, evolution, culture)
    Iota,
    /// Language & Linguistics (words, grammar, communication)
    Kappa,
    /// Philosophy & Ethics (thought, morality, logic)
    Lambda,
    /// Psychology & Neuroscience (mind, behavior, cognition)
    Mu,
    /// Programming & Systems (Rust, code, memory layout)
    Nu,
}

impl Domain {
    /// Get domain by name (case-insensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "alpha" | "math" | "mathematics" | "logic" => Some(Domain::Alpha),
            "beta" | "physics" | "chemistry" => Some(Domain::Beta),
            "gamma" | "astronomy" | "cosmology" | "space" => Some(Domain::Gamma),
            "delta" | "earth" | "environment" | "geography" | "climate" => Some(Domain::Delta),
            "epsilon" | "biology" | "medicine" | "health" => Some(Domain::Epsilon),
            "zeta" | "computer" | "cs" | "ai" | "ml" | "software" => Some(Domain::Zeta),
            "eta" | "engineering" | "tech" | "invention" => Some(Domain::Eta),
            "theta" | "economics" | "finance" | "market" | "economy" => Some(Domain::Theta),
            "iota" | "history" | "anthropology" | "civilization" => Some(Domain::Iota),
            "kappa" | "language" | "linguistics" | "words" => Some(Domain::Kappa),
            "lambda" | "philosophy" | "ethics" | "morality" => Some(Domain::Lambda),
            "mu" | "psychology" | "neuroscience" | "mind" | "brain" => Some(Domain::Mu),
            "nu" | "rust" | "rust_code" | "systems" | "programming" | "code" => Some(Domain::Nu),
            _ => None,
        }
    }

    /// Get Greek letter symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Domain::Alpha => "Α",
            Domain::Beta => "Β",
            Domain::Gamma => "Γ",
            Domain::Delta => "Δ",
            Domain::Epsilon => "Ε",
            Domain::Zeta => "Ζ",
            Domain::Eta => "Η",
            Domain::Theta => "Θ",
            Domain::Iota => "Ι",
            Domain::Kappa => "Κ",
            Domain::Lambda => "Λ",
            Domain::Mu => "Μ",
            Domain::Nu => "Ν",
        }
    }

    /// Get lowercase Greek letter
    pub fn symbol_lower(&self) -> &'static str {
        match self {
            Domain::Alpha => "α",
            Domain::Beta => "β",
            Domain::Gamma => "γ",
            Domain::Delta => "δ",
            Domain::Epsilon => "ε",
            Domain::Zeta => "ζ",
            Domain::Eta => "η",
            Domain::Theta => "θ",
            Domain::Iota => "ι",
            Domain::Kappa => "κ",
            Domain::Lambda => "λ",
            Domain::Mu => "μ",
            Domain::Nu => "ν",
        }
    }

    /// Get all domains
    pub fn all() -> Vec<Domain> {
        vec![
            Domain::Alpha,
            Domain::Beta,
            Domain::Gamma,
            Domain::Delta,
            Domain::Epsilon,
            Domain::Zeta,
            Domain::Eta,
            Domain::Theta,
            Domain::Iota,
            Domain::Kappa,
            Domain::Lambda,
            Domain::Mu,
            Domain::Nu,
        ]
    }

    /// Get domain description
    pub fn description(&self) -> &'static str {
        match self {
            Domain::Alpha => "Mathematics & Logic",
            Domain::Beta => "Physics & Chemistry",
            Domain::Gamma => "Astronomy & Cosmology",
            Domain::Delta => "Earth & Environment",
            Domain::Epsilon => "Biology & Medicine",
            Domain::Zeta => "Computer Science & AI",
            Domain::Eta => "Engineering & Technology",
            Domain::Theta => "Economics & Finance",
            Domain::Iota => "History & Anthropology",
            Domain::Kappa => "Language & Linguistics",
            Domain::Lambda => "Philosophy & Ethics",
            Domain::Mu => "Psychology & Neuroscience",
            Domain::Nu => "Programming & Systems",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub source: String,
    pub domain: Domain,
    pub created_at: u64,
    pub confidence: f64,
}

impl Fact {
    pub fn new(name: &str, value: f64, unit: &str, source: &str) -> Self {
        Fact {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            source: source.to_string(),
            domain: Domain::Alpha,
            created_at: now_epoch(),
            confidence: 1.0,
        }
    }

    /// Create fact with specific domain
    pub fn with_domain(name: &str, value: f64, unit: &str, source: &str, domain: Domain) -> Self {
        Fact {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            source: source.to_string(),
            domain,
            created_at: now_epoch(),
            confidence: 1.0,
        }
    }

    /// Create a dynamic fact (lower confidence, time-stamped)
    pub fn dynamic(
        name: &str,
        value: f64,
        unit: &str,
        source: &str,
        domain: Domain,
        confidence: f64,
    ) -> Self {
        Fact {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            source: source.to_string(),
            domain,
            created_at: now_epoch(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pub facts: Vec<Fact>,
    index: HashMap<String, usize>,
    domain_index: HashMap<Domain, Vec<usize>>,
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeBase {
    #[allow(clippy::vec_init_then_push)]
    pub fn new() -> Self {
        let mut facts = Vec::new();
        // --- Mathematical Constants ---
        facts.push(Fact::new("pi", std::f64::consts::PI, "", "math constant"));
        facts.push(Fact::new("e", std::f64::consts::E, "", "math constant"));
        facts.push(Fact::new(
            "sqrt2",
            std::f64::consts::SQRT_2,
            "",
            "math constant",
        ));
        facts.push(Fact::new(
            "ln2",
            std::f64::consts::LN_2,
            "",
            "math constant",
        ));
        facts.push(Fact::new(
            "ln10",
            std::f64::consts::LN_10,
            "",
            "math constant",
        ));
        facts.push(Fact::new("phi", 1.618033988749895, "", "golden ratio"));
        facts.push(Fact::new(
            "euler_mascheroni",
            0.5772156649015329,
            "",
            "math constant",
        ));

        // --- Rust / Systems (verified ABI, 64-bit; word = 8 bytes) ---
        // Source of truth mirrors proof/src/corpus/rust_seed.toml. Sizes are the
        // on-stack/inline footprint of the type, not heap payloads.
        facts.push(Fact::with_domain(
            "vec",
            24.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "string",
            24.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "slice",
            16.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "dyn",
            16.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "arc",
            16.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "rc",
            8.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "box",
            8.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "usize",
            8.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "isize",
            8.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "i8",
            1.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "i16",
            2.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "i32",
            4.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "i64",
            8.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "u8",
            1.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "u16",
            2.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "u32",
            4.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "u64",
            8.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "f32",
            4.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "f64",
            8.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "bool",
            1.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));
        facts.push(Fact::with_domain(
            "char",
            4.0,
            "bytes",
            "Rust std layout, 64-bit",
            Domain::Nu,
        ));

        // --- Physical Constants ---
        facts.push(Fact::new(
            "c",
            299792458.0,
            "m/s",
            "speed of light in vacuum",
        ));
        facts.push(Fact::new("g", 9.80665, "m/s^2", "standard gravity"));
        facts.push(Fact::new(
            "G",
            6.67430e-11,
            "N*m^2/kg^2",
            "gravitational constant",
        ));
        facts.push(Fact::new("h", 6.62607015e-34, "J*s", "Planck constant"));
        facts.push(Fact::new(
            "hbar",
            1.054571817e-34,
            "J*s",
            "reduced Planck constant",
        ));
        facts.push(Fact::new("kB", 1.380649e-23, "J/K", "Boltzmann constant"));
        facts.push(Fact::new("NA", 6.02214076e23, "1/mol", "Avogadro constant"));
        facts.push(Fact::new(
            "e_charge",
            1.602176634e-19,
            "C",
            "elementary charge",
        ));
        facts.push(Fact::new("eV", 1.602176634e-19, "J", "electron volt"));
        facts.push(Fact::new("me", 9.1093837015e-31, "kg", "electron mass"));
        facts.push(Fact::new("mp", 1.67262192369e-27, "kg", "proton mass"));
        facts.push(Fact::new(
            "mu0",
            1.25663706212e-6,
            "H/m",
            "vacuum permeability",
        ));
        facts.push(Fact::new(
            "eps0",
            8.8541878128e-12,
            "F/m",
            "vacuum permittivity",
        ));
        facts.push(Fact::new("R", 8.314462618, "J/(mol*K)", "gas constant"));
        facts.push(Fact::new(
            "sigma",
            5.670374419e-8,
            "W/(m^2*K^4)",
            "Stefan-Boltzmann constant",
        ));
        facts.push(Fact::new(
            "k_e",
            8.9875517923e9,
            "N*m^2/C^2",
            "Coulomb constant",
        ));

        // --- Fundamental Lengths ---
        facts.push(Fact::new(
            "bohr_radius",
            5.29177210903e-11,
            "m",
            "Bohr radius",
        ));
        facts.push(Fact::new(
            "compton_wavelength",
            2.42631023867e-12,
            "m",
            "Compton wavelength of electron",
        ));
        facts.push(Fact::new(
            "planck_length",
            1.616255e-35,
            "m",
            "Planck length",
        ));

        // --- Time ---
        facts.push(Fact::new("planck_time", 5.391247e-44, "s", "Planck time"));
        facts.push(Fact::new("year_seconds", 31556925.2, "s", "sidereal year"));
        facts.push(Fact::new("day_seconds", 86400.0, "s", "solar day"));
        facts.push(Fact::new("hour_seconds", 3600.0, "s", "hour"));

        // --- Density & Pressure ---
        facts.push(Fact::new(
            "water_density",
            1000.0,
            "kg/m^3",
            "density of water",
        ));
        facts.push(Fact::new(
            "air_density",
            1.225,
            "kg/m^3",
            "density of air at sea level",
        ));
        facts.push(Fact::new("atm", 101325.0, "Pa", "standard atmosphere"));
        facts.push(Fact::new(
            "speed_sound",
            343.0,
            "m/s",
            "speed of sound in air",
        ));

        // --- Earth ---
        facts.push(Fact::new("earth_mass", 5.972e24, "kg", "mass of Earth"));
        facts.push(Fact::new(
            "earth_radius",
            6.371e6,
            "m",
            "equatorial radius of Earth",
        ));
        facts.push(Fact::new(
            "earth_surface_area",
            5.1e14,
            "m^2",
            "surface area of Earth",
        ));
        facts.push(Fact::new(
            "earth_water",
            1.335e18,
            "m^3",
            "volume of water on Earth",
        ));
        facts.push(Fact::new(
            "earth_ocean",
            3.618e8,
            "km^2",
            "ocean surface area",
        ));
        facts.push(Fact::new(
            "earth_land",
            1.489e8,
            "km^2",
            "land surface area",
        ));
        facts.push(Fact::new(
            "earth_population",
            8045311447.0,
            "",
            "world population",
        ));

        // --- Moon ---
        facts.push(Fact::new("moon_mass", 7.342e22, "kg", "mass of Moon"));
        facts.push(Fact::new("moon_radius", 1737400.0, "m", "radius of Moon"));
        facts.push(Fact::new(
            "moon_distance",
            3.844e8,
            "m",
            "average distance to Moon",
        ));

        // --- Sun ---
        facts.push(Fact::new("sun_mass", 1.989e30, "kg", "mass of Sun"));
        facts.push(Fact::new("sun_radius", 6.957e8, "m", "radius of Sun"));
        facts.push(Fact::new(
            "sun_luminosity",
            3.828e26,
            "W",
            "solar luminosity",
        ));
        facts.push(Fact::new(
            "sun_temperature",
            5778.0,
            "K",
            "surface temperature of Sun",
        ));

        // --- Astronomy ---
        facts.push(Fact::new("au", 1.496e11, "m", "astronomical unit"));
        facts.push(Fact::new("light_year", 9.461e15, "m", "light year"));
        facts.push(Fact::new("parsec", 3.0857e16, "m", "parsec"));
        facts.push(Fact::new(
            "speed_earth_orbit",
            29783.0,
            "m/s",
            "Earth orbital speed",
        ));
        facts.push(Fact::new(
            "milky_way_diameter",
            8.74e20,
            "m",
            "Milky Way diameter",
        ));
        facts.push(Fact::new(
            "milky_way_stars",
            2.5e11,
            "",
            "estimated stars in Milky Way",
        ));
        facts.push(Fact::new(
            "observable_universe",
            8.8e26,
            "m",
            "radius of observable universe",
        ));
        facts.push(Fact::new(
            "cosmic_cmb",
            2.725,
            "K",
            "cosmic microwave background temperature",
        ));

        // --- Planets ---
        facts.push(Fact::new("jupiter_mass", 1.898e27, "kg", "mass of Jupiter"));
        facts.push(Fact::new(
            "jupiter_radius",
            69911000.0,
            "m",
            "equatorial radius of Jupiter",
        ));
        facts.push(Fact::new("saturn_mass", 5.683e26, "kg", "mass of Saturn"));
        facts.push(Fact::new(
            "saturn_radius",
            58232000.0,
            "m",
            "equatorial radius of Saturn",
        ));
        facts.push(Fact::new("mars_mass", 6.39e23, "kg", "mass of Mars"));
        facts.push(Fact::new(
            "mars_radius",
            3389500.0,
            "m",
            "equatorial radius of Mars",
        ));
        facts.push(Fact::new("venus_mass", 4.867e24, "kg", "mass of Venus"));
        facts.push(Fact::new(
            "venus_radius",
            6051800.0,
            "m",
            "equatorial radius of Venus",
        ));
        facts.push(Fact::new("mercury_mass", 3.285e23, "kg", "mass of Mercury"));
        facts.push(Fact::new(
            "mercury_radius",
            2439700.0,
            "m",
            "equatorial radius of Mercury",
        ));

        // --- Speed of Light ---
        facts.push(Fact::new(
            "c_mph",
            670616629.0,
            "mph",
            "speed of light in mph",
        ));
        facts.push(Fact::new(
            "c_knots",
            582749918.0,
            "knots",
            "speed of light in knots",
        ));

        // --- AI Model Parameters (actual trained sizes) ---
        facts.push(Fact::new(
            "gpt3_params",
            175.0,
            "billion",
            "GPT-3 parameter count",
        ));
        facts.push(Fact::new(
            "gpt4_params",
            1800.0,
            "billion",
            "GPT-4 estimated parameter count",
        ));
        facts.push(Fact::new(
            "llama2_70b_params",
            70.0,
            "billion",
            "LLaMA 2 70B parameter count",
        ));
        facts.push(Fact::new(
            "llama2_13b_params",
            13.0,
            "billion",
            "LLaMA 2 13B parameter count",
        ));
        facts.push(Fact::new(
            "llama2_7b_params",
            7.0,
            "billion",
            "LLaMA 2 7B parameter count",
        ));
        facts.push(Fact::new(
            "llama2_7b_tokens",
            2.0,
            "trillion",
            "LLaMA 2 7B training tokens",
        ));
        facts.push(Fact::new(
            "llama2_70b_tokens",
            2.0,
            "trillion",
            "LLaMA 2 70B training tokens",
        ));
        facts.push(Fact::new(
            "mistral_7b_params",
            7.3,
            "billion",
            "Mistral 7B parameter count",
        ));
        facts.push(Fact::new(
            "mixtral_8x7b_params",
            46.7,
            "billion",
            "Mixtral 8x7B parameter count",
        ));
        facts.push(Fact::new(
            "gemma_2b_params",
            2.5,
            "billion",
            "Gemma 2B parameter count",
        ));
        facts.push(Fact::new(
            "gemma_7b_params",
            7.8,
            "billion",
            "Gemma 7B parameter count",
        ));
        facts.push(Fact::new(
            "falcon_40b_params",
            40.0,
            "billion",
            "Falcon 40B parameter count",
        ));
        facts.push(Fact::new(
            "mpt_30b_params",
            30.0,
            "billion",
            "MPT-30B parameter count",
        ));
        facts.push(Fact::new(
            "phi_2_params",
            2.7,
            "billion",
            "Phi-2 parameter count",
        ));
        facts.push(Fact::new(
            "bert_large_params",
            340.0,
            "million",
            "BERT-Large parameter count",
        ));
        facts.push(Fact::new(
            "bert_base_params",
            110.0,
            "million",
            "BERT-Base parameter count",
        ));
        facts.push(Fact::new(
            "resnet50_params",
            25.6,
            "million",
            "ResNet-50 parameter count",
        ));
        facts.push(Fact::new(
            "resnet152_params",
            60.2,
            "million",
            "ResNet-152 parameter count",
        ));
        facts.push(Fact::new(
            "vit_base_params",
            86.0,
            "million",
            "ViT-Base parameter count",
        ));
        facts.push(Fact::new(
            "whisper_large_params",
            1550.0,
            "million",
            "Whisper Large parameter count",
        ));
        facts.push(Fact::new(
            "stable_diffusion_params",
            860.0,
            "million",
            "Stable Diffusion parameter count",
        ));
        facts.push(Fact::new(
            "human_brain_neurons",
            86.0,
            "billion",
            "estimated human brain neurons",
        ));
        facts.push(Fact::new(
            "human_brain_synapses",
            100.0,
            "trillion",
            "estimated human brain synapses",
        ));

        // --- GPU Specifications ---
        facts.push(Fact::new(
            "h100_transistors",
            80.0,
            "billion",
            "NVIDIA H100 transistor count",
        ));
        facts.push(Fact::new(
            "h100_fp64",
            33.5,
            "TFLOPS",
            "NVIDIA H100 FP64 performance",
        ));
        facts.push(Fact::new(
            "h100_fp32",
            66.9,
            "TFLOPS",
            "NVIDIA H100 FP32 performance",
        ));
        facts.push(Fact::new(
            "h100_fp16",
            989.4,
            "TFLOPS",
            "NVIDIA H100 FP16 Tensor Core",
        ));
        facts.push(Fact::new("h100_tdp", 700.0, "W", "NVIDIA H100 TDP"));
        facts.push(Fact::new("h100_hbm", 80.0, "GB", "NVIDIA H100 HBM3 memory"));
        facts.push(Fact::new(
            "a100_transistors",
            54.2,
            "billion",
            "NVIDIA A100 transistor count",
        ));
        facts.push(Fact::new(
            "a100_fp64",
            19.5,
            "TFLOPS",
            "NVIDIA A100 FP64 performance",
        ));
        facts.push(Fact::new(
            "a100_fp32",
            19.5,
            "TFLOPS",
            "NVIDIA A100 FP32 performance",
        ));
        facts.push(Fact::new("a100_tdp", 400.0, "W", "NVIDIA A100 TDP"));
        facts.push(Fact::new(
            "a100_hbm",
            80.0,
            "GB",
            "NVIDIA A100 HBM2e memory",
        ));
        facts.push(Fact::new(
            "a100_tensor_tf32",
            156.0,
            "TFLOPS",
            "NVIDIA A100 Tensor TF32",
        ));
        facts.push(Fact::new(
            "h100_tensor_tf32",
            989.4,
            "TFLOPS",
            "NVIDIA H100 Tensor TF32",
        ));
        facts.push(Fact::new(
            "h100_l2_cache",
            50.0,
            "MB",
            "NVIDIA H100 L2 cache",
        ));
        facts.push(Fact::new(
            "a100_l2_cache",
            40.0,
            "MB",
            "NVIDIA A100 L2 cache",
        ));
        facts.push(Fact::new(
            "h100_memory_bandwidth",
            3350.0,
            "GB/s",
            "NVIDIA H100 HBM3 bandwidth",
        ));
        facts.push(Fact::new(
            "a100_memory_bandwidth",
            2039.0,
            "GB/s",
            "NVIDIA A100 HBM2e bandwidth",
        ));
        facts.push(Fact::new(
            "v100_fp64",
            7.8,
            "TFLOPS",
            "NVIDIA V100 FP64 performance",
        ));
        facts.push(Fact::new(
            "v100_fp32",
            15.7,
            "TFLOPS",
            "NVIDIA V100 FP32 performance",
        ));
        facts.push(Fact::new("v100_tdp", 300.0, "W", "NVIDIA V100 TDP"));
        facts.push(Fact::new("v100_hbm", 32.0, "GB", "NVIDIA V100 HBM2 memory"));
        facts.push(Fact::new(
            "v100_tensor_tf32",
            125.0,
            "TFLOPS",
            "NVIDIA V100 Tensor TF32",
        ));
        facts.push(Fact::new(
            "rtx4090_transistors",
            76.3,
            "billion",
            "RTX 4090 transistor count",
        ));
        facts.push(Fact::new(
            "rtx4090_fp32",
            82.6,
            "TFLOPS",
            "RTX 4090 FP32 performance",
        ));
        facts.push(Fact::new("rtx4090_tdp", 450.0, "W", "RTX 4090 TDP"));
        facts.push(Fact::new("rtx4090_vram", 24.0, "GB", "RTX 4090 VRAM"));
        facts.push(Fact::new(
            "rtx3090_transistors",
            28.3,
            "billion",
            "RTX 3090 transistor count",
        ));
        facts.push(Fact::new(
            "rtx3090_fp32",
            35.6,
            "TFLOPS",
            "RTX 3090 FP32 performance",
        ));
        facts.push(Fact::new("rtx3090_tdp", 350.0, "W", "RTX 3090 TDP"));
        facts.push(Fact::new("rtx3090_vram", 24.0, "GB", "RTX 3090 VRAM"));
        facts.push(Fact::new(
            "tpu_v4_flop",
            275.0,
            "TFLOPS",
            "Google TPU v4 peak TFLOPS",
        ));
        facts.push(Fact::new(
            "tpu_v4_hbm",
            32.0,
            "GB",
            "Google TPU v4 HBM memory",
        ));
        facts.push(Fact::new(
            "mi300x_transistors",
            153.0,
            "billion",
            "AMD MI300X transistor count",
        ));
        facts.push(Fact::new(
            "mi300x_fp32",
            163.4,
            "TFLOPS",
            "AMD MI300X FP32 performance",
        ));
        facts.push(Fact::new(
            "mi300x_hbm",
            192.0,
            "GB",
            "AMD MI300X HBM3 memory",
        ));
        facts.push(Fact::new(
            "intel_max_transistors",
            100.0,
            "billion",
            "Intel Max Series GPU transistors",
        ));
        facts.push(Fact::new(
            "intel_max_fp32",
            52.0,
            "TFLOPS",
            "Intel Max Series GPU FP32",
        ));
        facts.push(Fact::new(
            "intel_max_hbm",
            128.0,
            "GB",
            "Intel Max Series GPU HBM",
        ));

        // --- Network & Bandwidth ---
        facts.push(Fact::new(
            "internet_backbone_bps",
            3.4e17,
            "bps",
            "estimated global internet backbone capacity",
        ));
        facts.push(Fact::new(
            "google_daily_searches",
            8.5e9,
            "",
            "Google daily search queries",
        ));
        facts.push(Fact::new(
            "google_daily_hours",
            360000.0,
            "years",
            "daily YouTube watch hours",
        ));
        facts.push(Fact::new(
            "facebook_daily_posts",
            2.0e9,
            "",
            "Facebook daily posts",
        ));
        facts.push(Fact::new(
            "twitter_daily_tweets",
            5.0e8,
            "",
            "Twitter daily tweets",
        ));
        facts.push(Fact::new(
            "cloudflare_daily_requests",
            2.5e13,
            "",
            "Cloudflare daily HTTP requests",
        ));
        facts.push(Fact::new(
            "aws_instances",
            1.0e6,
            "",
            "estimated AWS EC2 instances",
        ));
        facts.push(Fact::new("aws_regions", 31.0, "", "AWS cloud regions"));
        facts.push(Fact::new(
            "aws_edge_locations",
            600.0,
            "",
            "AWS CloudFront edge locations",
        ));

        // --- Cloud API Pricing ---
        facts.push(Fact::new(
            "gpt4_8k_input",
            0.03,
            "$/1k tokens",
            "GPT-4 8K input pricing",
        ));
        facts.push(Fact::new(
            "gpt4_8k_output",
            0.06,
            "$/1k tokens",
            "GPT-4 8K output pricing",
        ));
        facts.push(Fact::new(
            "gpt4_32k_input",
            0.06,
            "$/1k tokens",
            "GPT-4 32K input pricing",
        ));
        facts.push(Fact::new(
            "gpt4_32k_output",
            0.12,
            "$/1k tokens",
            "GPT-4 32K output pricing",
        ));
        facts.push(Fact::new(
            "gpt35turbo_input",
            0.0015,
            "$/1k tokens",
            "GPT-3.5 Turbo input pricing",
        ));
        facts.push(Fact::new(
            "gpt35turbo_output",
            0.002,
            "$/1k tokens",
            "GPT-3.5 Turbo output pricing",
        ));
        facts.push(Fact::new(
            "claude2_input",
            0.008,
            "$/1k tokens",
            "Claude 2 input pricing",
        ));
        facts.push(Fact::new(
            "claude2_output",
            0.024,
            "$/1k tokens",
            "Claude 2 output pricing",
        ));
        facts.push(Fact::new(
            "palm2_input",
            0.00025,
            "$/1k chars",
            "PaLM 2 input pricing",
        ));
        facts.push(Fact::new(
            "palm2_output",
            0.0005,
            "$/1k chars",
            "PaLM 2 output pricing",
        ));
        facts.push(Fact::new(
            "aws_bedrock_claude_input",
            0.008,
            "$/1k tokens",
            "AWS Bedrock Claude input",
        ));
        facts.push(Fact::new(
            "aws_bedrock_claude_output",
            0.024,
            "$/1k tokens",
            "AWS Bedrock Claude output",
        ));
        facts.push(Fact::new(
            "azure_openai_gpt4_input",
            0.03,
            "$/1k tokens",
            "Azure OpenAI GPT-4 input",
        ));
        facts.push(Fact::new(
            "azure_openai_gpt4_output",
            0.06,
            "$/1k tokens",
            "Azure OpenAI GPT-4 output",
        ));
        facts.push(Fact::new(
            "openai_embedding_ada",
            0.0001,
            "$/1k tokens",
            "OpenAI text-embedding-ada-002",
        ));
        facts.push(Fact::new(
            "cohere_embed_input",
            0.0001,
            "$/1k tokens",
            "Cohere Embed input pricing",
        ));
        facts.push(Fact::new(
            "anthropic_batch_input",
            0.003,
            "$/1k tokens",
            "Anthropic Batch API input",
        ));
        facts.push(Fact::new(
            "anthropic_batch_output",
            0.015,
            "$/1k tokens",
            "Anthropic Batch API output",
        ));
        facts.push(Fact::new(
            "aws_ec2_p3_8xl",
            12.24,
            "$/hour",
            "p3.8xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_p4d_24xl",
            32.77,
            "$/hour",
            "p4d.24xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_p5_48xl",
            98.32,
            "$/hour",
            "p5.48xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_g5_2xl",
            3.01,
            "$/hour",
            "g5.2xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_g5_4xl",
            5.67,
            "$/hour",
            "g5.4xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_g5_12xl",
            19.83,
            "$/hour",
            "g5.12xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_g5_48xl",
            16.29,
            "$/hour",
            "g5.48xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_trn1_2xl",
            1.34,
            "$/hour",
            "trn1.2xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_ec2_trn1_32xl",
            21.5,
            "$/hour",
            "trn1.32xlarge on-demand",
        ));
        facts.push(Fact::new(
            "aws_s3_standard",
            0.023,
            "$/GB/month",
            "S3 Standard storage",
        ));
        facts.push(Fact::new(
            "aws_s3_ia",
            0.0125,
            "$/GB/month",
            "S3 Infrequent Access",
        ));
        facts.push(Fact::new(
            "aws_data_transfer_out",
            0.09,
            "$/GB",
            "AWS data transfer out",
        ));
        facts.push(Fact::new("azure_nc6", 0.9, "$/hour", "Azure NC6 on-demand"));
        facts.push(Fact::new(
            "gcp_a2_highgpu_1g",
            3.67,
            "$/hour",
            "a2-highgpu-1g on-demand",
        ));
        facts.push(Fact::new(
            "gcp_a2_highgpu_8g",
            29.39,
            "$/hour",
            "a2-highgpu-8g on-demand",
        ));
        facts.push(Fact::new(
            "lambda_8gpu_8h",
            12.12,
            "$/hour",
            "Lambda 8x A100 80GB 8h",
        ));
        facts.push(Fact::new(
            "lambda_8gpu_24h",
            11.06,
            "$/hour",
            "Lambda 8x A100 80GB 24h",
        ));
        facts.push(Fact::new(
            "coreweave_a100_80gb",
            1.64,
            "$/hour",
            "CoreWeave A100 80GB",
        ));
        facts.push(Fact::new(
            "oracle_a100_40gb",
            32.0,
            "$/hour",
            "Oracle A100 40GB instance",
        ));
        facts.push(Fact::new(
            "h100_cloud_gpu_hour",
            35.0,
            "$/hour",
            "estimated H100 cloud GPU",
        ));

        // --- ML Benchmark Scores ---
        facts.push(Fact::new("gpt4_mmlu", 86.4, "%", "GPT-4 MMLU score"));
        facts.push(Fact::new("gpt35_mmlu", 70.0, "%", "GPT-3.5 MMLU score"));
        facts.push(Fact::new(
            "llama2_70b_mmlu",
            68.9,
            "%",
            "LLaMA 2 70B MMLU score",
        ));
        facts.push(Fact::new(
            "llama2_13b_mmlu",
            54.8,
            "%",
            "LLaMA 2 13B MMLU score",
        ));
        facts.push(Fact::new(
            "llama2_7b_mmlu",
            45.3,
            "%",
            "LLaMA 2 7B MMLU score",
        ));
        facts.push(Fact::new(
            "llama3_70b_mmlu",
            82.0,
            "%",
            "LLaMA 3 70B MMLU score",
        ));
        facts.push(Fact::new(
            "llama3_8b_mmlu",
            68.4,
            "%",
            "LLaMA 3 8B MMLU score",
        ));
        facts.push(Fact::new(
            "mixtral_8x7b_mmlu",
            70.6,
            "%",
            "Mixtral 8x7B MMLU score",
        ));
        facts.push(Fact::new(
            "mistral_7b_mmlu",
            62.5,
            "%",
            "Mistral 7B MMLU score",
        ));
        facts.push(Fact::new("gemma_7b_mmlu", 64.3, "%", "Gemma 7B MMLU score"));
        facts.push(Fact::new("gemma_2b_mmlu", 42.3, "%", "Gemma 2B MMLU score"));
        facts.push(Fact::new(
            "gpt4_hellaswag",
            95.3,
            "%",
            "GPT-4 HellaSwag score",
        ));
        facts.push(Fact::new(
            "llama2_70b_hellaswag",
            85.3,
            "%",
            "LLaMA 2 70B HellaSwag",
        ));
        facts.push(Fact::new(
            "gpt4_arc_challenge",
            96.3,
            "%",
            "GPT-4 ARC-Challenge score",
        ));
        facts.push(Fact::new(
            "gpt4_winogrande",
            87.5,
            "%",
            "GPT-4 Winogrande score",
        ));
        facts.push(Fact::new(
            "llama2_70b_gsm8k",
            56.8,
            "%",
            "LLaMA 2 70B GSM8K score",
        ));
        facts.push(Fact::new("gpt4_gsm8k", 92.0, "%", "GPT-4 GSM8K score"));
        facts.push(Fact::new(
            "human_gsm8k",
            100.0,
            "%",
            "human performance GSM8K",
        ));
        facts.push(Fact::new(
            "gpt4_human eval",
            67.0,
            "%",
            "GPT-4 HumanEval pass@1",
        ));
        facts.push(Fact::new(
            "gpt4_truthfulqa",
            59.0,
            "%",
            "GPT-4 TruthfulQA score",
        ));
        facts.push(Fact::new("human_mmlu", 89.8, "%", "human performance MMLU"));
        facts.push(Fact::new(
            "human_hellaswag",
            95.6,
            "%",
            "human performance HellaSwag",
        ));
        facts.push(Fact::new(
            "human_arc_challenge",
            96.3,
            "%",
            "human performance ARC-Challenge",
        ));
        facts.push(Fact::new(
            "human_winogrande",
            94.2,
            "%",
            "human performance Winogrande",
        ));
        facts.push(Fact::new(
            "human_truthfulqa",
            54.6,
            "%",
            "human performance TruthfulQA",
        ));
        facts.push(Fact::new(
            "resnet50_imagenet",
            76.1,
            "%",
            "ResNet-50 ImageNet top-1",
        ));
        facts.push(Fact::new(
            "vit_b_16_imagenet",
            77.9,
            "%",
            "ViT-B/16 ImageNet top-1",
        ));
        facts.push(Fact::new(
            "vit_l_14_imagenet",
            85.3,
            "%",
            "ViT-L/14 ImageNet top-1",
        ));
        facts.push(Fact::new(
            "gpt4_sat_math",
            94.0,
            "%",
            "GPT-4 SAT Math score",
        ));
        facts.push(Fact::new(
            "gpt4_ap_bio",
            94.8,
            "%",
            "GPT-4 AP Biology score",
        ));
        facts.push(Fact::new(
            "gpt4_ap_calc",
            88.0,
            "%",
            "GPT-4 AP Calculus BC score",
        ));
        facts.push(Fact::new(
            "gpt4_bar_exam",
            90.0,
            "%",
            "GPT-4 bar exam score percentile",
        ));
        facts.push(Fact::new(
            "gpt4_usabo",
            100.0,
            "%",
            "GPT-4 USABO semifinalist score",
        ));
        facts.push(Fact::new("gpt4_usnco", 93.0, "%", "GPT-4 USNCO score"));
        facts.push(Fact::new(
            "gpt4_lsat",
            88.0,
            "%",
            "GPT-4 LSAT score percentile",
        ));

        // --- Finance ---
        facts.push(Fact::new(
            "sp500_avg_return",
            10.5,
            "%",
            "S&P 500 historical average annual return",
        ));
        facts.push(Fact::new(
            "sp500_dividend_yield",
            1.5,
            "%",
            "S&P 500 average dividend yield",
        ));
        facts.push(Fact::new(
            "us_10yr_treasury",
            4.5,
            "%",
            "US 10-year Treasury yield",
        ));
        facts.push(Fact::new(
            "us_30yr_mortgage",
            7.5,
            "%",
            "US 30-year fixed mortgage rate",
        ));
        facts.push(Fact::new(
            "fed_funds_rate",
            5.5,
            "%",
            "Federal funds target rate",
        ));
        facts.push(Fact::new("us_inflation", 3.0, "%", "US CPI inflation rate"));
        facts.push(Fact::new(
            "us_gdp",
            25.46,
            "trillion dollars",
            "US nominal GDP",
        ));
        facts.push(Fact::new(
            "china_gdp",
            17.96,
            "trillion dollars",
            "China nominal GDP",
        ));
        facts.push(Fact::new(
            "japan_gdp",
            4.23,
            "trillion dollars",
            "Japan nominal GDP",
        ));
        facts.push(Fact::new(
            "germany_gdp",
            4.07,
            "trillion dollars",
            "Germany nominal GDP",
        ));
        facts.push(Fact::new(
            "india_gdp",
            3.39,
            "trillion dollars",
            "India nominal GDP",
        ));
        facts.push(Fact::new(
            "uk_gdp",
            3.07,
            "trillion dollars",
            "UK nominal GDP",
        ));
        facts.push(Fact::new(
            "france_gdp",
            2.78,
            "trillion dollars",
            "France nominal GDP",
        ));
        facts.push(Fact::new(
            "global_gdp",
            100.0,
            "trillion dollars",
            "global nominal GDP",
        ));
        facts.push(Fact::new(
            "global_market_cap",
            109.0,
            "trillion dollars",
            "global stock market cap",
        ));
        facts.push(Fact::new(
            "us_market_cap",
            50.0,
            "trillion dollars",
            "US stock market cap",
        ));
        facts.push(Fact::new(
            "gold_price_oz",
            2000.0,
            "USD",
            "gold price per ounce",
        ));
        facts.push(Fact::new(
            "bitcoin_max_supply",
            21.0,
            "million",
            "Bitcoin maximum supply",
        ));
        facts.push(Fact::new(
            "eth_block_reward",
            2.0,
            "ETH",
            "Ethereum post-merge block reward",
        ));
        facts.push(Fact::new(
            "eth_block_time",
            12.0,
            "seconds",
            "Ethereum block time",
        ));
        facts.push(Fact::new(
            "visa_daily_txns",
            720.0,
            "million",
            "Visa daily transactions",
        ));
        facts.push(Fact::new(
            "visa_peak_tps",
            65000.0,
            "TPS",
            "Visa peak transactions per second",
        ));
        facts.push(Fact::new(
            "btc_peak_tps",
            7.0,
            "TPS",
            "Bitcoin peak transactions per second",
        ));
        facts.push(Fact::new(
            "eth_peak_tps",
            30.0,
            "TPS",
            "Ethereum peak transactions per second",
        ));
        facts.push(Fact::new(
            "solana_peak_tps",
            65000.0,
            "TPS",
            "Solana peak transactions per second",
        ));
        facts.push(Fact::new(
            "nyse_mkt_cap",
            28.0,
            "trillion dollars",
            "NYSE total market cap",
        ));
        facts.push(Fact::new(
            "nasdaq_mkt_cap",
            22.0,
            "trillion dollars",
            "NASDAQ total market cap",
        ));
        facts.push(Fact::new(
            "global_ad_spend",
            740.0,
            "billion dollars",
            "global advertising spend",
        ));
        facts.push(Fact::new(
            "amazon_revenue",
            574.0,
            "billion dollars",
            "Amazon 2023 annual revenue",
        ));
        facts.push(Fact::new(
            "apple_revenue",
            383.0,
            "billion dollars",
            "Apple 2023 annual revenue",
        ));
        facts.push(Fact::new(
            "google_revenue",
            307.0,
            "billion dollars",
            "Google 2023 annual revenue",
        ));
        facts.push(Fact::new(
            "microsoft_revenue",
            212.0,
            "billion dollars",
            "Microsoft 2023 annual revenue",
        ));
        facts.push(Fact::new(
            "nvidia_revenue",
            60.9,
            "billion dollars",
            "NVIDIA 2024 fiscal year revenue",
        ));
        facts.push(Fact::new(
            "tesla_revenue",
            96.8,
            "billion dollars",
            "Tesla 2023 annual revenue",
        ));
        facts.push(Fact::new(
            "meta_revenue",
            134.9,
            "billion dollars",
            "Meta 2023 annual revenue",
        ));
        facts.push(Fact::new(
            "amazon_employees",
            1540000.0,
            "",
            "Amazon employee count",
        ));
        facts.push(Fact::new(
            "apple_employees",
            164000.0,
            "",
            "Apple employee count",
        ));
        facts.push(Fact::new(
            "google_employees",
            182000.0,
            "",
            "Alphabet employee count",
        ));
        facts.push(Fact::new(
            "microsoft_employees",
            221000.0,
            "",
            "Microsoft employee count",
        ));
        facts.push(Fact::new(
            "nvidia_employees",
            29600.0,
            "",
            "NVIDIA employee count",
        ));
        facts.push(Fact::new(
            "tesla_employees",
            140473.0,
            "",
            "Tesla employee count",
        ));
        facts.push(Fact::new(
            "meta_employees",
            67317.0,
            "",
            "Meta employee count",
        ));
        facts.push(Fact::new(
            "global_smartphone_users",
            6930000000.0,
            "",
            "global smartphone users",
        ));
        facts.push(Fact::new(
            "global_internet_users",
            5300000000.0,
            "",
            "global internet users",
        ));
        facts.push(Fact::new(
            "global_social_media_users",
            4950000000.0,
            "",
            "global social media users",
        ));
        facts.push(Fact::new(
            "global_5g_connections",
            2000000000.0,
            "",
            "global 5G connections",
        ));
        facts.push(Fact::new(
            "spacex_falcon9_payload_leo",
            22800.0,
            "kg",
            "Falcon 9 payload to LEO",
        ));
        facts.push(Fact::new(
            "spacex_starship_payload_leo",
            100000.0,
            "kg",
            "Starship payload to LEO",
        ));
        facts.push(Fact::new(
            "spacex_falcon9_cost",
            67.0,
            "million dollars",
            "Falcon 9 launch cost",
        ));
        facts.push(Fact::new(
            "global_tech_rnd",
            900.0,
            "billion dollars",
            "global tech company R&D spend",
        ));
        facts.push(Fact::new(
            "us_federal_spending",
            6.1,
            "trillion dollars",
            "US federal spending 2024",
        ));
        facts.push(Fact::new(
            "us_federal_revenue",
            4.4,
            "trillion dollars",
            "US federal revenue 2024",
        ));
        facts.push(Fact::new(
            "us_federal_debt",
            34.0,
            "trillion dollars",
            "US national debt",
        ));
        facts.push(Fact::new(
            "global_debt",
            307.0,
            "trillion dollars",
            "global total debt",
        ));
        facts.push(Fact::new(
            "global_derivatives_notional",
            714.0,
            "trillion dollars",
            "global derivatives notional",
        ));
        facts.push(Fact::new(
            "global_hedge_fund_aum",
            4.3,
            "trillion dollars",
            "global hedge fund AUM",
        ));
        facts.push(Fact::new(
            "vc_annual_investment",
            350.0,
            "billion dollars",
            "global VC annual investment",
        ));
        facts.push(Fact::new(
            "ai_market_2030",
            1800.0,
            "billion dollars",
            "projected AI market size 2030",
        ));
        facts.push(Fact::new(
            "ai_market_cagr",
            37.3,
            "%",
            "AI market CAGR 2023-2030",
        ));
        facts.push(Fact::new(
            "global_erc_aum",
            13.0,
            "trillion dollars",
            "global equity research coverage AUM",
        ));

        // --- Historical Dates ---
        facts.push(Fact::new(
            "us_independence",
            1776.0,
            "AD",
            "US Declaration of Independence",
        ));
        facts.push(Fact::new(
            "french_revolution",
            1789.0,
            "AD",
            "French Revolution",
        ));
        facts.push(Fact::new(
            "us_constitution",
            1789.0,
            "AD",
            "US Constitution ratified",
        ));
        facts.push(Fact::new(
            "world_war_1_start",
            1914.0,
            "AD",
            "World War I start",
        ));
        facts.push(Fact::new(
            "world_war_1_end",
            1918.0,
            "AD",
            "World War I end",
        ));
        facts.push(Fact::new(
            "world_war_2_start",
            1939.0,
            "AD",
            "World War II start",
        ));
        facts.push(Fact::new(
            "world_war_2_end",
            1945.0,
            "AD",
            "World War II end",
        ));
        facts.push(Fact::new(
            "un_founded",
            1945.0,
            "AD",
            "United Nations founded",
        ));
        facts.push(Fact::new(
            "moon_landing",
            1969.0,
            "AD",
            "Apollo 11 Moon landing",
        ));
        facts.push(Fact::new(
            "berlin_wall_fall",
            1989.0,
            "AD",
            "Fall of the Berlin Wall",
        ));
        facts.push(Fact::new(
            "world_wide_web",
            1991.0,
            "AD",
            "World Wide Web launched",
        ));
        facts.push(Fact::new(
            "human_genome",
            2003.0,
            "AD",
            "Human Genome Project completed",
        ));
        facts.push(Fact::new("iphone_launch", 2007.0, "AD", "iPhone launch"));
        facts.push(Fact::new(
            "bitcoin_genesis",
            2009.0,
            "AD",
            "Bitcoin genesis block",
        ));
        facts.push(Fact::new(
            "chatgpt_launch",
            2022.0,
            "AD",
            "ChatGPT launched",
        ));
        facts.push(Fact::new("gpt4_launch", 2023.0, "AD", "GPT-4 launched"));
        facts.push(Fact::new("ai_act_eu", 2024.0, "AD", "EU AI Act adopted"));
        facts.push(Fact::new(
            "pythagorean_theorem",
            -530.0,
            "BC",
            "Pythagorean theorem proved",
        ));
        facts.push(Fact::new(
            "euclid_elements",
            -300.0,
            "BC",
            "Euclid's Elements written",
        ));
        facts.push(Fact::new(
            "zero_invented",
            628.0,
            "AD",
            "Zero as a number first used",
        ));
        facts.push(Fact::new(
            "printing_press",
            1440.0,
            "AD",
            "Gutenberg printing press",
        ));
        facts.push(Fact::new(
            "steam_engine",
            1712.0,
            "AD",
            "Newcomen steam engine",
        ));
        facts.push(Fact::new(
            "electricity_grid",
            1882.0,
            "AD",
            "First commercial electricity grid",
        ));
        facts.push(Fact::new(
            "powered_flight",
            1903.0,
            "AD",
            "Wright brothers first flight",
        ));
        facts.push(Fact::new(
            "television_broadcast",
            1928.0,
            "AD",
            "First electronic TV broadcast",
        ));
        facts.push(Fact::new(
            "transistor_invented",
            1947.0,
            "AD",
            "Transistor invented at Bell Labs",
        ));
        facts.push(Fact::new("arpanet", 1969.0, "AD", "ARPANET first message"));
        facts.push(Fact::new(
            "personal_computer",
            1975.0,
            "AD",
            "Altair 8800 personal computer",
        ));
        facts.push(Fact::new("ibm_pc", 1981.0, "AD", "IBM PC released"));
        facts.push(Fact::new(
            "world_wide_web_invented",
            1989.0,
            "AD",
            "Tim Berners-Lee invents WWW",
        ));
        facts.push(Fact::new("google_founded", 1998.0, "AD", "Google founded"));
        facts.push(Fact::new(
            "facebook_founded",
            2004.0,
            "AD",
            "Facebook founded",
        ));
        facts.push(Fact::new(
            "twitter_founded",
            2006.0,
            "AD",
            "Twitter founded",
        ));
        facts.push(Fact::new(
            "facebook_ai",
            2013.0,
            "AD",
            "Facebook AI Research founded",
        ));
        facts.push(Fact::new(
            "alpha_go",
            2016.0,
            "AD",
            "AlphaGo defeats Lee Sedol",
        ));
        facts.push(Fact::new(
            "transformer_paper",
            2017.0,
            "AD",
            "Attention Is All You Need published",
        ));
        facts.push(Fact::new("gpt3_release", 2020.0, "AD", "GPT-3 released"));
        facts.push(Fact::new("dall_e", 2021.0, "AD", "DALL-E released"));
        facts.push(Fact::new(
            "stable_diffusion",
            2022.0,
            "AD",
            "Stable Diffusion released",
        ));
        facts.push(Fact::new(
            "llama_release",
            2023.0,
            "AD",
            "Meta releases LLaMA",
        ));
        facts.push(Fact::new(
            "llama2_release",
            2023.0,
            "AD",
            "Meta releases LLaMA 2",
        ));
        facts.push(Fact::new(
            "llama3_release",
            2024.0,
            "AD",
            "Meta releases LLaMA 3",
        ));
        facts.push(Fact::new(
            "mixtral_release",
            2024.0,
            "AD",
            "Mistral releases Mixtral",
        ));
        facts.push(Fact::new(
            "gemini_release",
            2023.0,
            "AD",
            "Google releases Gemini",
        ));
        facts.push(Fact::new("grok_release", 2023.0, "AD", "xAI releases Grok"));
        facts.push(Fact::new(
            "claude_release",
            2023.0,
            "AD",
            "Anthropic releases Claude",
        ));
        facts.push(Fact::new(
            "eu_ai_act_passage",
            2024.0,
            "AD",
            "EU AI Act final passage",
        ));
        facts.push(Fact::new(
            "bretton_woods",
            1944.0,
            "AD",
            "Bretton Woods Agreement",
        ));
        facts.push(Fact::new(
            "imf_founded",
            1945.0,
            "AD",
            "International Monetary Fund founded",
        ));
        facts.push(Fact::new(
            "world_bank_founded",
            1944.0,
            "AD",
            "World Bank founded",
        ));
        facts.push(Fact::new(
            "nixon_shock",
            1971.0,
            "AD",
            "Nixon ends gold standard",
        ));
        facts.push(Fact::new(
            "black_scholes",
            1973.0,
            "AD",
            "Black-Scholes model published",
        ));
        facts.push(Fact::new(
            "lehman_collapse",
            2008.0,
            "AD",
            "Lehman Brothers collapse",
        ));

        // --- Programming Facts ---
        facts.push(Fact::new(
            "first_computer",
            1945.0,
            "AD",
            "ENIAC first general-purpose computer",
        ));
        facts.push(Fact::new(
            "fortran_release",
            1957.0,
            "AD",
            "FORTRAN released",
        ));
        facts.push(Fact::new(
            "c_language",
            1972.0,
            "AD",
            "C language developed",
        ));
        facts.push(Fact::new(
            "unix_created",
            1969.0,
            "AD",
            "Unix developed at Bell Labs",
        ));
        facts.push(Fact::new("ibm_pc_dos", 1981.0, "AD", "IBM PC-DOS released"));
        facts.push(Fact::new(
            "windows_1_0",
            1985.0,
            "AD",
            "Windows 1.0 released",
        ));
        facts.push(Fact::new(
            "linus_torvalds",
            1991.0,
            "AD",
            "Linux kernel started by Linus Torvalds",
        ));
        facts.push(Fact::new(
            "python_release",
            1991.0,
            "AD",
            "Python 0.9.0 released",
        ));
        facts.push(Fact::new("java_release", 1995.0, "AD", "Java 1.0 released"));
        facts.push(Fact::new(
            "google_founded_year",
            1998.0,
            "AD",
            "Google founded",
        ));
        facts.push(Fact::new("github_founded", 2008.0, "AD", "GitHub launched"));
        facts.push(Fact::new("docker_release", 2013.0, "AD", "Docker released"));
        facts.push(Fact::new("rust_1_0", 2015.0, "AD", "Rust 1.0 released"));
        facts.push(Fact::new(
            "typescript_release",
            2012.0,
            "AD",
            "TypeScript released",
        ));
        facts.push(Fact::new("webassembly", 2017.0, "AD", "WebAssembly MVP"));
        facts.push(Fact::new("zig_release", 2016.0, "AD", "Zig 0.1.0 released"));
        facts.push(Fact::new("go_release", 2012.0, "AD", "Go 1.0 released"));
        facts.push(Fact::new(
            "swift_release",
            2014.0,
            "AD",
            "Swift released by Apple",
        ));
        facts.push(Fact::new(
            "kotlin_release",
            2016.0,
            "AD",
            "Kotlin 1.0 released",
        ));
        facts.push(Fact::new(
            "elixir_release",
            2012.0,
            "AD",
            "Elixir 1.0 released",
        ));

        // --- Populations ---
        facts.push(Fact::new(
            "china_population",
            1412.0,
            "million",
            "China population",
        ));
        facts.push(Fact::new(
            "india_population",
            1408.0,
            "million",
            "India population",
        ));
        facts.push(Fact::new(
            "us_population",
            331.0,
            "million",
            "United States population",
        ));
        facts.push(Fact::new(
            "indonesia_population",
            275.0,
            "million",
            "Indonesia population",
        ));
        facts.push(Fact::new(
            "pakistan_population",
            220.0,
            "million",
            "Pakistan population",
        ));
        facts.push(Fact::new(
            "brazil_population",
            214.0,
            "million",
            "Brazil population",
        ));
        facts.push(Fact::new(
            "nigeria_population",
            218.0,
            "million",
            "Nigeria population",
        ));
        facts.push(Fact::new(
            "bangladesh_population",
            170.0,
            "million",
            "Bangladesh population",
        ));
        facts.push(Fact::new(
            "russia_population",
            144.0,
            "million",
            "Russia population",
        ));
        facts.push(Fact::new(
            "mexico_population",
            128.0,
            "million",
            "Mexico population",
        ));
        facts.push(Fact::new(
            "japan_population",
            125.0,
            "million",
            "Japan population",
        ));
        facts.push(Fact::new(
            "ethiopia_population",
            120.0,
            "million",
            "Ethiopia population",
        ));
        facts.push(Fact::new(
            "philippines_population",
            113.0,
            "million",
            "Philippines population",
        ));
        facts.push(Fact::new(
            "egypt_population",
            104.0,
            "million",
            "Egypt population",
        ));
        facts.push(Fact::new(
            "vietnam_population",
            98.0,
            "million",
            "Vietnam population",
        ));
        facts.push(Fact::new(
            "dr_congo_population",
            100.0,
            "million",
            "DR Congo population",
        ));
        facts.push(Fact::new(
            "turkey_population",
            85.0,
            "million",
            "Turkey population",
        ));
        facts.push(Fact::new(
            "iran_population",
            87.0,
            "million",
            "Iran population",
        ));
        facts.push(Fact::new(
            "germany_population",
            84.0,
            "million",
            "Germany population",
        ));
        facts.push(Fact::new(
            "thailand_population",
            72.0,
            "million",
            "Thailand population",
        ));
        facts.push(Fact::new(
            "uk_population",
            67.0,
            "million",
            "United Kingdom population",
        ));
        facts.push(Fact::new(
            "france_population",
            68.0,
            "million",
            "France population",
        ));
        facts.push(Fact::new(
            "italy_population",
            59.0,
            "million",
            "Italy population",
        ));
        facts.push(Fact::new(
            "south_africa_population",
            60.0,
            "million",
            "South Africa population",
        ));
        facts.push(Fact::new(
            "south_korea_population",
            52.0,
            "million",
            "South Korea population",
        ));
        facts.push(Fact::new(
            "colombia_population",
            51.0,
            "million",
            "Colombia population",
        ));
        facts.push(Fact::new(
            "kenya_population",
            55.0,
            "million",
            "Kenya population",
        ));
        facts.push(Fact::new(
            "spain_population",
            47.0,
            "million",
            "Spain population",
        ));
        facts.push(Fact::new(
            "argentina_population",
            46.0,
            "million",
            "Argentina population",
        ));
        facts.push(Fact::new(
            "ukraine_population",
            44.0,
            "million",
            "Ukraine population",
        ));
        facts.push(Fact::new(
            "algeria_population",
            45.0,
            "million",
            "Algeria population",
        ));
        facts.push(Fact::new(
            "sudan_population",
            47.0,
            "million",
            "Sudan population",
        ));
        facts.push(Fact::new(
            "iraq_population",
            43.0,
            "million",
            "Iraq population",
        ));
        facts.push(Fact::new(
            "afghanistan_population",
            41.0,
            "million",
            "Afghanistan population",
        ));
        facts.push(Fact::new(
            "poland_population",
            38.0,
            "million",
            "Poland population",
        ));
        facts.push(Fact::new(
            "canada_population",
            39.0,
            "million",
            "Canada population",
        ));
        facts.push(Fact::new(
            "morocco_population",
            37.0,
            "million",
            "Morocco population",
        ));
        facts.push(Fact::new(
            "saudi_arabia_population",
            36.0,
            "million",
            "Saudi Arabia population",
        ));
        facts.push(Fact::new(
            "uzbekistan_population",
            35.0,
            "million",
            "Uzbekistan population",
        ));
        facts.push(Fact::new(
            "peru_population",
            34.0,
            "million",
            "Peru population",
        ));
        facts.push(Fact::new(
            "angola_population",
            35.0,
            "million",
            "Angola population",
        ));
        facts.push(Fact::new(
            "mozambique_population",
            33.0,
            "million",
            "Mozambique population",
        ));
        facts.push(Fact::new(
            "ghana_population",
            33.0,
            "million",
            "Ghana population",
        ));
        facts.push(Fact::new(
            "yemen_population",
            33.0,
            "million",
            "Yemen population",
        ));
        facts.push(Fact::new(
            "nepal_population",
            30.0,
            "million",
            "Nepal population",
        ));
        facts.push(Fact::new(
            "venezuela_population",
            28.0,
            "million",
            "Venezuela population",
        ));
        facts.push(Fact::new(
            "madagascar_population",
            29.0,
            "million",
            "Madagascar population",
        ));
        facts.push(Fact::new(
            "cameroon_population",
            28.0,
            "million",
            "Cameroon population",
        ));
        facts.push(Fact::new(
            "ivory_coast_population",
            28.0,
            "million",
            "Ivory Coast population",
        ));
        facts.push(Fact::new(
            "australia_population",
            26.0,
            "million",
            "Australia population",
        ));
        facts.push(Fact::new(
            "north_korea_population",
            26.0,
            "million",
            "North Korea population",
        ));
        facts.push(Fact::new(
            "taiwan_population",
            24.0,
            "million",
            "Taiwan population",
        ));
        facts.push(Fact::new(
            "sri_lanka_population",
            22.0,
            "million",
            "Sri Lanka population",
        ));
        facts.push(Fact::new(
            "malaysia_population",
            33.0,
            "million",
            "Malaysia population",
        ));
        facts.push(Fact::new(
            "zimbabwe_population",
            16.0,
            "million",
            "Zimbabwe population",
        ));
        facts.push(Fact::new(
            "zambia_population",
            20.0,
            "million",
            "Zambia population",
        ));
        facts.push(Fact::new(
            "senegal_population",
            18.0,
            "million",
            "Senegal population",
        ));
        facts.push(Fact::new(
            "chad_population",
            18.0,
            "million",
            "Chad population",
        ));
        facts.push(Fact::new(
            "somalia_population",
            18.0,
            "million",
            "Somalia population",
        ));
        facts.push(Fact::new(
            "guinea_population",
            14.0,
            "million",
            "Guinea population",
        ));
        facts.push(Fact::new(
            "rwanda_population",
            14.0,
            "million",
            "Rwanda population",
        ));
        facts.push(Fact::new(
            "benin_population",
            13.0,
            "million",
            "Benin population",
        ));
        facts.push(Fact::new(
            "tunisia_population",
            12.0,
            "million",
            "Tunisia population",
        ));
        facts.push(Fact::new(
            "burundi_population",
            13.0,
            "million",
            "Burundi population",
        ));
        facts.push(Fact::new(
            "south_sudan_population",
            11.0,
            "million",
            "South Sudan population",
        ));
        facts.push(Fact::new(
            "togo_population",
            9.0,
            "million",
            "Togo population",
        ));
        facts.push(Fact::new(
            "sierra_leone_population",
            8.0,
            "million",
            "Sierra Leone population",
        ));
        facts.push(Fact::new(
            "laos_population",
            8.0,
            "million",
            "Laos population",
        ));
        facts.push(Fact::new(
            "paraguay_population",
            7.0,
            "million",
            "Paraguay population",
        ));
        facts.push(Fact::new(
            "libya_population",
            7.0,
            "million",
            "Libya population",
        ));
        facts.push(Fact::new(
            "bulgaria_population",
            6.5,
            "million",
            "Bulgaria population",
        ));
        facts.push(Fact::new(
            "nicaragua_population",
            7.0,
            "million",
            "Nicaragua population",
        ));
        facts.push(Fact::new(
            "serbia_population",
            6.6,
            "million",
            "Serbia population",
        ));
        facts.push(Fact::new(
            "honduras_population",
            10.0,
            "million",
            "Honduras population",
        ));
        facts.push(Fact::new(
            "el_salvador_population",
            6.5,
            "million",
            "El Salvador population",
        ));
        facts.push(Fact::new(
            "jordan_population",
            11.0,
            "million",
            "Jordan population",
        ));
        facts.push(Fact::new(
            "turkmenistan_population",
            6.3,
            "million",
            "Turkmenistan population",
        ));
        facts.push(Fact::new(
            "uruguay_population",
            3.5,
            "million",
            "Uruguay population",
        ));
        facts.push(Fact::new(
            "bosnia_population",
            3.2,
            "million",
            "Bosnia and Herzegovina population",
        ));
        facts.push(Fact::new(
            "mongolia_population",
            3.3,
            "million",
            "Mongolia population",
        ));
        facts.push(Fact::new(
            "armenia_population",
            3.0,
            "million",
            "Armenia population",
        ));
        facts.push(Fact::new(
            "jamaica_population",
            2.8,
            "million",
            "Jamaica population",
        ));
        facts.push(Fact::new(
            "qatar_population",
            2.9,
            "million",
            "Qatar population",
        ));
        facts.push(Fact::new(
            "albania_population",
            2.8,
            "million",
            "Albania population",
        ));
        facts.push(Fact::new(
            "puerto_rico_population",
            3.2,
            "million",
            "Puerto Rico population",
        ));
        facts.push(Fact::new(
            "ireland_population",
            5.1,
            "million",
            "Ireland population",
        ));
        facts.push(Fact::new(
            "finland_population",
            5.5,
            "million",
            "Finland population",
        ));
        facts.push(Fact::new(
            "new_zealand_population",
            5.1,
            "million",
            "New Zealand population",
        ));
        facts.push(Fact::new(
            "norway_population",
            5.5,
            "million",
            "Norway population",
        ));
        facts.push(Fact::new(
            "singapore_population",
            5.9,
            "million",
            "Singapore population",
        ));
        facts.push(Fact::new(
            "denmark_population",
            5.9,
            "million",
            "Denmark population",
        ));
        facts.push(Fact::new(
            "sweden_population",
            10.4,
            "million",
            "Sweden population",
        ));
        facts.push(Fact::new(
            "switzerland_population",
            8.8,
            "million",
            "Switzerland population",
        ));
        facts.push(Fact::new(
            "austria_population",
            9.1,
            "million",
            "Austria population",
        ));
        facts.push(Fact::new(
            "belgium_population",
            11.6,
            "million",
            "Belgium population",
        ));
        facts.push(Fact::new(
            "portugal_population",
            10.3,
            "million",
            "Portugal population",
        ));
        facts.push(Fact::new(
            "greece_population",
            10.4,
            "million",
            "Greece population",
        ));
        facts.push(Fact::new(
            "czech_republic_population",
            10.8,
            "million",
            "Czech Republic population",
        ));
        facts.push(Fact::new(
            "romania_population",
            19.0,
            "million",
            "Romania population",
        ));
        facts.push(Fact::new(
            "hungary_population",
            9.7,
            "million",
            "Hungary population",
        ));
        facts.push(Fact::new(
            "belarus_population",
            9.4,
            "million",
            "Belarus population",
        ));
        facts.push(Fact::new(
            "cuba_population",
            11.3,
            "million",
            "Cuba population",
        ));
        facts.push(Fact::new(
            "greece_ancient",
            -800.0,
            "BC",
            "Ancient Greek civilization",
        ));

        // --- Geography ---
        facts.push(Fact::new(
            "mount_everest",
            8848.86,
            "m",
            "Mount Everest elevation",
        ));
        facts.push(Fact::new("dead_sea", -430.5, "m", "Dead Sea elevation"));
        facts.push(Fact::new(
            "pacific_ocean_area",
            1.684e14,
            "m^2",
            "Pacific Ocean surface area",
        ));
        facts.push(Fact::new(
            "atlantic_ocean_area",
            1.0646e14,
            "m^2",
            "Atlantic Ocean surface area",
        ));
        facts.push(Fact::new(
            "indian_ocean_area",
            7.056e13,
            "m^2",
            "Indian Ocean surface area",
        ));
        facts.push(Fact::new(
            "arctic_ocean_area",
            1.406e13,
            "m^2",
            "Arctic Ocean surface area",
        ));
        facts.push(Fact::new(
            "southern_ocean_area",
            2.196e13,
            "m^2",
            "Southern Ocean surface area",
        ));
        facts.push(Fact::new(
            "amazon_river_length",
            6400.0,
            "km",
            "Amazon River length",
        ));
        facts.push(Fact::new(
            "nile_river_length",
            6650.0,
            "km",
            "Nile River length",
        ));
        facts.push(Fact::new(
            "yangtze_river_length",
            6300.0,
            "km",
            "Yangtze River length",
        ));
        facts.push(Fact::new(
            "mississippi_river_length",
            6275.0,
            "km",
            "Mississippi River length",
        ));
        facts.push(Fact::new(
            "yenesie_river_length",
            5539.0,
            "km",
            "Yenisey River length",
        ));
        facts.push(Fact::new(
            "yellow_river_length",
            5464.0,
            "km",
            "Yellow River length",
        ));
        facts.push(Fact::new(
            "ob_irtysh_river_length",
            5410.0,
            "km",
            "Ob-Irtysh River length",
        ));
        facts.push(Fact::new(
            "parana_river_length",
            4880.0,
            "km",
            "Parana River length",
        ));
        facts.push(Fact::new(
            "congo_river_length",
            4700.0,
            "km",
            "Congo River length",
        ));
        facts.push(Fact::new(
            "volga_river_length",
            3530.0,
            "km",
            "Volga River length",
        ));
        facts.push(Fact::new(
            "danube_river_length",
            2860.0,
            "km",
            "Danube River length",
        ));
        facts.push(Fact::new(
            "ganges_river_length",
            2704.0,
            "km",
            "Ganges River length",
        ));
        facts.push(Fact::new(
            "euphrates_river_length",
            2800.0,
            "km",
            "Euphrates River length",
        ));
        facts.push(Fact::new(
            "tigris_river_length",
            1900.0,
            "km",
            "Tigris River length",
        ));
        facts.push(Fact::new(
            "rhine_river_length",
            1233.0,
            "km",
            "Rhine River length",
        ));
        facts.push(Fact::new(
            "thames_river_length",
            346.0,
            "km",
            "Thames River length",
        ));
        facts.push(Fact::new(
            "seine_river_length",
            777.0,
            "km",
            "Seine River length",
        ));
        facts.push(Fact::new(
            "amur_river_length",
            4444.0,
            "km",
            "Amur River length",
        ));
        facts.push(Fact::new(
            "mekong_river_length",
            4350.0,
            "km",
            "Mekong River length",
        ));
        facts.push(Fact::new(
            "zambezi_river_length",
            2574.0,
            "km",
            "Zambezi River length",
        ));
        facts.push(Fact::new(
            "brahmaputra_river_length",
            3848.0,
            "km",
            "Brahmaputra River length",
        ));
        facts.push(Fact::new(
            "dnieper_river_length",
            2201.0,
            "km",
            "Dnieper River length",
        ));
        facts.push(Fact::new(
            "baltic_sea_area",
            3.77e14,
            "m^2",
            "Baltic Sea surface area",
        ));
        facts.push(Fact::new(
            "mediterranean_area",
            2.5e14,
            "m^2",
            "Mediterranean Sea surface area",
        ));
        facts.push(Fact::new(
            "black_sea_area",
            4.364e14,
            "m^2",
            "Black Sea surface area",
        ));
        facts.push(Fact::new(
            "caspian_sea_area",
            3.71e14,
            "m^2",
            "Caspian Sea surface area",
        ));
        facts.push(Fact::new(
            "red_sea_area",
            4.38e14,
            "m^2",
            "Red Sea surface area",
        ));
        facts.push(Fact::new(
            "caribbean_sea_area",
            2.754e15,
            "m^2",
            "Caribbean Sea surface area",
        ));
        facts.push(Fact::new(
            "east_china_sea_area",
            1.249e15,
            "m^2",
            "East China Sea surface area",
        ));
        facts.push(Fact::new(
            "south_china_sea_area",
            3.5e15,
            "m^2",
            "South China Sea surface area",
        ));
        facts.push(Fact::new(
            "bering_sea_area",
            2.33e15,
            "m^2",
            "Bering Sea surface area",
        ));
        facts.push(Fact::new(
            "gulf_mexico_area",
            1.55e15,
            "m^2",
            "Gulf of Mexico surface area",
        ));
        facts.push(Fact::new(
            "hudson_bay_area",
            1.23e15,
            "m^2",
            "Hudson Bay surface area",
        ));
        facts.push(Fact::new(
            "great_lakes_area",
            2.44e14,
            "m^2",
            "Great Lakes surface area",
        ));
        facts.push(Fact::new(
            "baikal_depth",
            1642.0,
            "m",
            "Lake Baikal maximum depth",
        ));
        facts.push(Fact::new(
            "tanganyika_depth",
            1470.0,
            "m",
            "Lake Tanganyika maximum depth",
        ));
        facts.push(Fact::new(
            "vostok_depth",
            900.0,
            "m",
            "Lake Vostok maximum depth",
        ));
        facts.push(Fact::new(
            "superior_depth",
            406.0,
            "m",
            "Lake Superior maximum depth",
        ));
        facts.push(Fact::new(
            "huron_depth",
            229.0,
            "m",
            "Lake Huron maximum depth",
        ));
        facts.push(Fact::new(
            "michigan_depth",
            281.0,
            "m",
            "Lake Michigan maximum depth",
        ));
        facts.push(Fact::new(
            "erie_depth",
            64.0,
            "m",
            "Lake Erie maximum depth",
        ));
        facts.push(Fact::new(
            "ontario_depth",
            244.0,
            "m",
            "Lake Ontario maximum depth",
        ));
        facts.push(Fact::new(
            "toba_capacity",
            240.0,
            "km^3",
            "Lake Toba water capacity",
        ));
        facts.push(Fact::new(
            "toba_area",
            1130.0,
            "km^2",
            "Lake Toba surface area",
        ));
        facts.push(Fact::new(
            "toba_depth",
            505.0,
            "m",
            "Lake Toba maximum depth",
        ));
        facts.push(Fact::new(
            "toba_age",
            74000.0,
            "years",
            "Toba eruption years ago",
        ));
        facts.push(Fact::new(
            "sahara_area",
            9200000.0,
            "km^2",
            "Sahara Desert area",
        ));
        facts.push(Fact::new(
            "arabian_desert_area",
            2330000.0,
            "km^2",
            "Arabian Desert area",
        ));
        facts.push(Fact::new(
            "gobi_area",
            1295000.0,
            "km^2",
            "Gobi Desert area",
        ));
        facts.push(Fact::new(
            "kalahari_area",
            900000.0,
            "km^2",
            "Kalahari Desert area",
        ));
        facts.push(Fact::new(
            "patagonian_area",
            673000.0,
            "km^2",
            "Patagonian Desert area",
        ));
        facts.push(Fact::new(
            "great_victoria_area",
            348750.0,
            "km^2",
            "Great Victoria Desert area",
        ));
        facts.push(Fact::new(
            "chinese_wall_length",
            21196.0,
            "km",
            "Great Wall of China length",
        ));
        facts.push(Fact::new(
            "panama_canal_length",
            82.0,
            "km",
            "Panama Canal length",
        ));
        facts.push(Fact::new(
            "suez_canal_length",
            193.3,
            "km",
            "Suez Canal length",
        ));
        facts.push(Fact::new(
            "kansai_airport_area",
            511.0,
            "hectares",
            "Kansai International Airport area",
        ));
        facts.push(Fact::new(
            "palm_jumeirah_area",
            560.0,
            "hectares",
            "Palm Jumeirah area",
        ));
        facts.push(Fact::new(
            "three_gorges_dam_length",
            2335.0,
            "m",
            "Three Gorges Dam length",
        ));
        facts.push(Fact::new(
            "three_gorges_dam_height",
            181.0,
            "m",
            "Three Gorges Dam height",
        ));
        facts.push(Fact::new(
            "burj_khalifa_height",
            828.0,
            "m",
            "Burj Khalifa height",
        ));
        facts.push(Fact::new(
            "shanghai_tower_height",
            632.0,
            "m",
            "Shanghai Tower height",
        ));
        facts.push(Fact::new(
            "abraj_al_bait_height",
            601.0,
            "m",
            "Abraj Al-Bait height",
        ));
        facts.push(Fact::new(
            "ping_an_height",
            599.0,
            "m",
            "Ping An Finance Centre height",
        ));
        facts.push(Fact::new(
            "lotte_world_tower_height",
            554.5,
            "m",
            "Lotte World Tower height",
        ));
        facts.push(Fact::new(
            "one_wtc_height",
            541.3,
            "m",
            "One World Trade Center height",
        ));
        facts.push(Fact::new(
            "taipei_101_height",
            509.2,
            "m",
            "Taipei 101 height",
        ));
        facts.push(Fact::new(
            "shanghai_world_height",
            492.0,
            "m",
            "Shanghai World Financial Center height",
        ));
        facts.push(Fact::new(
            "internationalCommerce_height",
            484.0,
            "m",
            "International Commerce Centre height",
        ));
        facts.push(Fact::new(
            "sears_tower_height",
            442.1,
            "m",
            "Willis (Sears) Tower height",
        ));
        facts.push(Fact::new(
            "petronas_towers_height",
            451.9,
            "m",
            "Petronas Towers height",
        ));

        // --- Science Facts ---
        facts.push(Fact::new(
            "speed_light_km_s",
            299792.458,
            "km/s",
            "speed of light in km/s",
        ));
        facts.push(Fact::new(
            "speed_light_mph",
            670616629.0,
            "mph",
            "speed of light in mph",
        ));
        facts.push(Fact::new(
            "speed_light_knots",
            582749918.0,
            "knots",
            "speed of light in knots",
        ));
        facts.push(Fact::new(
            "speed_light_ft_s",
            983571056.0,
            "ft/s",
            "speed of light in ft/s",
        ));
        facts.push(Fact::new(
            "speed_light_round_earth",
            0.133,
            "s",
            "light time around Earth equator",
        ));
        facts.push(Fact::new(
            "speed_light_moon",
            1.282,
            "s",
            "light time to Moon",
        ));
        facts.push(Fact::new(
            "speed_light_sun",
            499.0,
            "s",
            "light time to Sun",
        ));
        facts.push(Fact::new(
            "speed_light_mars",
            752.0,
            "s",
            "light time to Mars at closest approach",
        ));
        facts.push(Fact::new(
            "speed_light_jupiter",
            2160.0,
            "s",
            "light time to Jupiter",
        ));
        facts.push(Fact::new(
            "speed_light_proxima",
            150480.0,
            "s",
            "light time to Proxima Centauri",
        ));
        facts.push(Fact::new(
            "speed_light_andromeda",
            7.5e12,
            "s",
            "light time to Andromeda Galaxy",
        ));

        // --- Chemistry ---
        facts.push(Fact::new(
            "atomic_mass_unit",
            1.66054e-27,
            "kg",
            "atomic mass unit",
        ));
        facts.push(Fact::new(
            "faraday_constant",
            96485.33212,
            "C/mol",
            "Faraday constant",
        ));
        facts.push(Fact::new(
            "standard_state_potential",
            1.42,
            "V",
            "H2/O2 standard potential",
        ));
        facts.push(Fact::new(
            "water_molecular_weight",
            18.01528,
            "g/mol",
            "water molecular weight",
        ));
        facts.push(Fact::new(
            "co2_molecular_weight",
            44.009,
            "g/mol",
            "CO2 molecular weight",
        ));
        facts.push(Fact::new(
            "o2_molecular_weight",
            31.998,
            "g/mol",
            "O2 molecular weight",
        ));
        facts.push(Fact::new(
            "n2_molecular_weight",
            28.014,
            "g/mol",
            "N2 molecular weight",
        ));
        facts.push(Fact::new(
            "h2_molecular_weight",
            2.016,
            "g/mol",
            "H2 molecular weight",
        ));
        facts.push(Fact::new(
            "ch4_molecular_weight",
            16.04,
            "g/mol",
            "CH4 molecular weight",
        ));
        facts.push(Fact::new(
            "glucose_molecular_weight",
            180.156,
            "g/mol",
            "glucose molecular weight",
        ));
        facts.push(Fact::new(
            "ethanol_molecular_weight",
            46.07,
            "g/mol",
            "ethanol molecular weight",
        ));
        facts.push(Fact::new(
            "sodium_chloride_mw",
            58.44,
            "g/mol",
            "NaCl molecular weight",
        ));
        facts.push(Fact::new(
            "iron_atomic_mass",
            55.845,
            "u",
            "iron atomic mass",
        ));
        facts.push(Fact::new(
            "carbon12_mass",
            12.0,
            "u",
            "carbon-12 atomic mass",
        ));
        facts.push(Fact::new(
            "hydrogen_boiling",
            20.271,
            "K",
            "hydrogen boiling point",
        ));
        facts.push(Fact::new(
            "helium_boiling",
            4.222,
            "K",
            "helium boiling point",
        ));
        facts.push(Fact::new(
            "nitrogen_boiling",
            77.355,
            "K",
            "nitrogen boiling point",
        ));
        facts.push(Fact::new(
            "oxygen_boiling",
            90.188,
            "K",
            "oxygen boiling point",
        ));
        facts.push(Fact::new(
            "water_boiling",
            373.15,
            "K",
            "water boiling point at 1 atm",
        ));
        facts.push(Fact::new(
            "water_freezing",
            273.15,
            "K",
            "water freezing point",
        ));
        facts.push(Fact::new(
            "gold_melting",
            1337.33,
            "K",
            "gold melting point",
        ));
        facts.push(Fact::new("iron_melting", 1811.0, "K", "iron melting point"));
        facts.push(Fact::new(
            "tungsten_melting",
            3695.0,
            "K",
            "tungsten melting point",
        ));
        facts.push(Fact::new(
            "diamond_carbon",
            12.011,
            "u",
            "carbon atomic mass",
        ));

        // --- Unit Conversions ---
        facts.push(Fact::new("mile_km", 1.60934, "km", "miles to kilometers"));
        facts.push(Fact::new("yard_m", 0.9144, "m", "yards to meters"));
        facts.push(Fact::new("inch_cm", 2.54, "cm", "inches to centimeters"));
        facts.push(Fact::new("pound_kg", 0.453592, "kg", "pounds to kilograms"));
        facts.push(Fact::new("ounce_gram", 28.3495, "g", "ounces to grams"));
        facts.push(Fact::new(
            "gallon_liter",
            3.78541,
            "L",
            "US gallons to liters",
        ));
        facts.push(Fact::new(
            "fahrenheit_celsius_factor",
            0.5556,
            "",
            "F to C conversion factor",
        ));
        facts.push(Fact::new(
            "fahrenheit_celsius_offset",
            -17.7778,
            "",
            "F to C offset",
        ));
        facts.push(Fact::new(
            "kelvin_celsius_offset",
            -273.15,
            "",
            "K to C offset",
        ));
        facts.push(Fact::new("atm_psi", 14.696, "psi", "atmospheres to PSI"));
        facts.push(Fact::new(
            "horsepower_watts",
            745.7,
            "W",
            "horsepower to watts",
        ));
        facts.push(Fact::new("btu_joules", 1055.06, "J", "BTU to joules"));
        facts.push(Fact::new(
            " calorie_joules",
            4.184,
            "J",
            "thermochemical calorie to joules",
        ));
        facts.push(Fact::new(
            "nautical_mile_m",
            1852.0,
            "m",
            "nautical mile to meters",
        ));
        facts.push(Fact::new(
            "lightyear_km",
            9.461e12,
            "km",
            "light-year to kilometers",
        ));
        facts.push(Fact::new(
            "parsec_au",
            206265.0,
            "AU",
            "parsec to astronomical units",
        ));
        facts.push(Fact::new(
            "parsec_ly",
            3.26156,
            "ly",
            "parsec to light-years",
        ));
        facts.push(Fact::new(
            "parsec_km",
            3.0857e13,
            "km",
            "parsec to kilometers",
        ));
        facts.push(Fact::new(
            "acre_sqm",
            4046.86,
            "m^2",
            "acres to square meters",
        ));
        facts.push(Fact::new(
            "hectare_sqm",
            10000.0,
            "m^2",
            "hectares to square meters",
        ));
        facts.push(Fact::new(
            "barrel_liters",
            158.987,
            "L",
            "oil barrel to liters",
        ));
        facts.push(Fact::new(
            "troy_ounce_grams",
            31.1035,
            "g",
            "troy ounce to grams",
        ));
        facts.push(Fact::new(
            "grain_milligrams",
            64.79891,
            "mg",
            "grain to milligrams",
        ));
        facts.push(Fact::new("stone_kg", 6.35029, "kg", "stone to kilograms"));
        facts.push(Fact::new(
            "hand_cm",
            10.16,
            "cm",
            "hand (horse height) to centimeters",
        ));
        facts.push(Fact::new("fathom_m", 1.8288, "m", "fathom to meters"));
        facts.push(Fact::new("rod_m", 5.0292, "m", "rod to meters"));
        facts.push(Fact::new("chain_m", 20.1168, "m", "chain to meters"));
        facts.push(Fact::new("furlong_m", 201.168, "m", "furlong to meters"));
        facts.push(Fact::new("league_km", 4.828, "km", "league to kilometers"));
        facts.push(Fact::new(
            "tbsp_ml",
            14.7868,
            "mL",
            "tablespoon to milliliters",
        ));
        facts.push(Fact::new(
            "tsp_ml",
            4.92892,
            "mL",
            "teaspoon to milliliters",
        ));
        facts.push(Fact::new("cup_ml", 236.588, "mL", "cup to milliliters"));
        facts.push(Fact::new(
            "pint_ml",
            473.176,
            "mL",
            "US pint to milliliters",
        ));
        facts.push(Fact::new("quart_l", 0.946353, "L", "US quart to liters"));

        // --- Social Impact & Development ---
        facts.push(Fact::new(
            "global_population",
            8045311447.0,
            "",
            "world population 2024",
        ));
        facts.push(Fact::new(
            "extreme_poverty_rate",
            8.5,
            "%",
            "global extreme poverty rate",
        ));
        facts.push(Fact::new(
            "literacy_rate",
            87.0,
            "%",
            "global adult literacy rate",
        ));
        facts.push(Fact::new(
            "life_expectancy_global",
            73.4,
            "years",
            "global average life expectancy",
        ));
        facts.push(Fact::new(
            "child_mortality_rate",
            37.0,
            "per 1000",
            "under-5 mortality rate",
        ));
        facts.push(Fact::new(
            "internet_penetration",
            64.4,
            "%",
            "global internet penetration",
        ));
        facts.push(Fact::new(
            "mobile_penetration",
            105.0,
            "%",
            "global mobile subscriptions per 100",
        ));
        facts.push(Fact::new(
            "co2_emissions_global",
            36.8,
            "Gt",
            "annual global CO2 emissions",
        ));
        facts.push(Fact::new(
            "renewable_energy_share",
            13.0,
            "%",
            "global renewable energy share",
        ));
        facts.push(Fact::new(
            "gini_index_global",
            38.0,
            "",
            "global Gini inequality index",
        ));

        // --- Economics & Finance ---
        facts.push(Fact::new(
            "global_gdp_nominal",
            105.0,
            "trillion dollars",
            "global nominal GDP 2024",
        ));
        facts.push(Fact::new(
            "global_gdp_ppp",
            165.0,
            "trillion dollars",
            "global GDP PPP 2024",
        ));
        facts.push(Fact::new(
            "us_gdp_per_capita",
            76330.0,
            "USD",
            "US GDP per capita",
        ));
        facts.push(Fact::new(
            "global_trade_volume",
            28.0,
            "trillion dollars",
            "global trade in goods",
        ));
        facts.push(Fact::new(
            "global_debt_gdp",
            336.0,
            "%",
            "global debt to GDP ratio",
        ));
        facts.push(Fact::new(
            "us_national_debt",
            34.0,
            "trillion dollars",
            "US national debt",
        ));
        facts.push(Fact::new(
            "global_m2_money",
            100.0,
            "trillion dollars",
            "global M2 money supply",
        ));
        facts.push(Fact::new(
            "gold_reserves_global",
            36000.0,
            "tonnes",
            "global central bank gold reserves",
        ));
        facts.push(Fact::new(
            "oil_price_brent",
            82.0,
            "USD/barrel",
            "Brent crude oil price",
        ));
        facts.push(Fact::new(
            "natural_gas_price",
            2.5,
            "USD/MMBtu",
            "Henry Hub natural gas price",
        ));

        // --- Health & Education ---
        facts.push(Fact::new(
            "healthcare_spending_global",
            9.8,
            "trillion dollars",
            "global healthcare spending",
        ));
        facts.push(Fact::new(
            "education_spending_global",
            6.5,
            "trillion dollars",
            "global education spending",
        ));
        facts.push(Fact::new(
            "research_spending_global",
            2.4,
            "trillion dollars",
            "global R&D spending",
        ));
        facts.push(Fact::new(
            "physician_density",
            18.0,
            "per 10000",
            "global physician density",
        ));
        facts.push(Fact::new(
            "hospital_beds",
            27.0,
            "per 10000",
            "global hospital beds per capita",
        ));
        facts.push(Fact::new(
            "clean_water_access",
            74.0,
            "%",
            "global access to clean water",
        ));
        facts.push(Fact::new(
            "sanitation_access",
            57.0,
            "%",
            "global access to improved sanitation",
        ));

        // --- Technology & Innovation ---
        facts.push(Fact::new(
            "global_r_and_d_spending",
            2.4,
            "trillion dollars",
            "global R&D spending",
        ));
        facts.push(Fact::new(
            "patent_filings_global",
            3.5,
            "million",
            "annual global patent filings",
        ));
        facts.push(Fact::new(
            "internet_users_global",
            5.3,
            "billion",
            "global internet users",
        ));
        facts.push(Fact::new(
            "smartphone_users_global",
            6.8,
            "billion",
            "global smartphone users",
        ));
        facts.push(Fact::new(
            "cloud_market_size",
            600.0,
            "billion dollars",
            "global cloud computing market",
        ));
        facts.push(Fact::new(
            "ai_market_size",
            200.0,
            "billion dollars",
            "global AI market size",
        ));
        facts.push(Fact::new(
            "cybercrime_cost",
            8.0,
            "trillion dollars",
            "annual global cybercrime cost",
        ));

        // --- Tanto Merged Facts: Car Prices (θ Economics) ---
        facts.push(Fact::with_domain(
            "price_tesla_model3",
            38990.0,
            "USD",
            "Tesla.com 2024 base price",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "price_camry",
            26420.0,
            "USD",
            "Toyota.com 2024 LE base price",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "price_camry_hybrid",
            28250.0,
            "USD",
            "Toyota 2024 LE hybrid base",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "price_civic",
            23950.0,
            "USD",
            "Honda 2024 LX base price",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "price_corolla",
            22050.0,
            "USD",
            "Toyota 2024 LE base price",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "price_prius",
            27500.0,
            "USD",
            "Toyota 2024 base price",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "mpg_camry",
            32.0,
            "mpg",
            "EPA 2024 Camry LE combined",
            Domain::Eta,
        ));
        facts.push(Fact::with_domain(
            "mpg_camry_hybrid",
            51.0,
            "mpg",
            "EPA 2024 Camry Hybrid LE combined",
            Domain::Eta,
        ));
        facts.push(Fact::with_domain(
            "mpg_civic",
            36.0,
            "mpg",
            "EPA 2024 Civic LX combined",
            Domain::Eta,
        ));
        facts.push(Fact::with_domain(
            "mpg_corolla",
            35.0,
            "mpg",
            "EPA 2024 Corolla LE combined",
            Domain::Eta,
        ));
        facts.push(Fact::with_domain(
            "mpg_prius",
            57.0,
            "mpg",
            "EPA 2024 Prius combined",
            Domain::Eta,
        ));
        facts.push(Fact::with_domain(
            "mpk_tesla_model3",
            4.1,
            "mi/kWh",
            "EPA 2024 Model 3 RWD efficiency",
            Domain::Eta,
        ));
        facts.push(Fact::with_domain(
            "gas_price_us",
            3.15,
            "USD/gal",
            "EIA 2024 US avg regular",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "elec_price_us",
            0.16,
            "USD/kWh",
            "EIA 2024 US avg residential",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "elec_price_ca",
            0.30,
            "USD/kWh",
            "EIA 2024 California residential",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "elec_price_wa",
            0.11,
            "USD/kWh",
            "EIA 2024 Washington residential",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "maint_ev",
            500.0,
            "USD/yr",
            "Consumer Reports EV avg maintenance",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "maint_ice",
            800.0,
            "USD/yr",
            "Consumer Reports gas car avg maintenance",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "maint_hybrid",
            700.0,
            "USD/yr",
            "Consumer Reports hybrid avg maintenance",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "resale_tesla",
            0.50,
            "ratio",
            "Kelley Blue Book 5-yr avg resale value",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "resale_camry",
            0.55,
            "ratio",
            "KBB 5-yr Camry resale value",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "resale_civic",
            0.52,
            "ratio",
            "KBB 5-yr Civic resale value",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "resale_avg",
            0.45,
            "ratio",
            "KBB 5-yr industry average resale",
            Domain::Theta,
        ));

        // --- Tanto Merged Facts: Speed Limits (δ Earth) ---
        facts.push(Fact::with_domain(
            "speed_limit_hwy",
            65.0,
            "mph",
            "US interstate default speed limit",
            Domain::Delta,
        ));
        facts.push(Fact::with_domain(
            "speed_limit_city",
            30.0,
            "mph",
            "US city default speed limit",
            Domain::Delta,
        ));

        // --- Tanto Merged Facts: AI Model Specs (ζ CS/AI) ---
        facts.push(Fact::with_domain(
            "gpt4_context",
            128.0,
            "K tokens",
            "GPT-4 Turbo context window",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "gpt4o_input",
            2.50,
            "USD/Mtok",
            "GPT-4o input price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "gpt4o_output",
            10.0,
            "USD/Mtok",
            "GPT-4o output price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "gpt4o_mini_input",
            0.15,
            "USD/Mtok",
            "GPT-4o-mini input price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "gpt4o_mini_output",
            0.60,
            "USD/Mtok",
            "GPT-4o-mini output price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "claude_sonnet_input",
            3.0,
            "USD/Mtok",
            "Claude 3.5 Sonnet input price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "claude_sonnet_output",
            15.0,
            "USD/Mtok",
            "Claude 3.5 Sonnet output price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "claude_haiku_input",
            0.25,
            "USD/Mtok",
            "Claude 3.5 Haiku input price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "claude_haiku_output",
            1.25,
            "USD/Mtok",
            "Claude 3.5 Haiku output price per million tokens",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "embedding_dim_ada",
            1536.0,
            "dims",
            "text-embedding-ada-002 dimensions",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "embedding_dim_3large",
            3072.0,
            "dims",
            "text-embedding-3-large dimensions",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "token_rate_gpt4o",
            84.0,
            "tok/s",
            "GPT-4o typical output tokens per second",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "token_rate_haiku",
            75.0,
            "tok/s",
            "Claude 3.5 Haiku typical output tokens/second",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "token_rate_sonnet",
            55.0,
            "tok/s",
            "Claude 3.5 Sonnet typical output tokens/second",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "ram_macbook",
            18.0,
            "GB",
            "Apple M-series unified memory typical",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "ram_developer",
            32.0,
            "GB",
            "Typical developer workstation RAM",
            Domain::Zeta,
        ));

        // --- Tanto Merged Facts: Hardware (ζ CS/AI) ---
        facts.push(Fact::with_domain(
            "vram_h100",
            80.0,
            "GB",
            "NVIDIA H100 SXM VRAM",
            Domain::Zeta,
        ));
        facts.push(Fact::with_domain(
            "flops_h100",
            2000.0,
            "TFLOPS",
            "NVIDIA H100 FP8 sparse TFLOPS",
            Domain::Zeta,
        ));

        // --- Tanto Merged Facts: Material Densities (η Engineering) ---
        facts.push(Fact::with_domain(
            "steel_density",
            7850.0,
            "kg/m^3",
            "Mild steel density",
            Domain::Eta,
        ));
        facts.push(Fact::with_domain(
            "aluminum_density",
            2700.0,
            "kg/m^3",
            "Pure aluminum density",
            Domain::Eta,
        ));

        // --- Tanto Merged Facts: Human Speeds (μ Psychology) ---
        facts.push(Fact::with_domain(
            "human_walking",
            1.4,
            "m/s",
            "Average human walking speed",
            Domain::Mu,
        ));
        facts.push(Fact::with_domain(
            "human_running",
            5.0,
            "m/s",
            "Average human jogging speed",
            Domain::Mu,
        ));

        // --- Tanto Merged Facts: Laws & Principles (λ Philosophy) ---
        facts.push(Fact::with_domain(
            "pareto_principle",
            80.0,
            "%",
            "Pareto: 80% of effects from 20% of causes",
            Domain::Lambda,
        ));
        facts.push(Fact::with_domain(
            "hackers_law",
            10.0,
            "%",
            "Nielsen: 10% of users generate 90% of content",
            Domain::Lambda,
        ));
        facts.push(Fact::with_domain(
            "goodhart_law",
            1.0,
            "ratio",
            "When a metric becomes a target it ceases to be a good metric",
            Domain::Lambda,
        ));
        facts.push(Fact::with_domain(
            "conway_law",
            1.0,
            "ratio",
            "Systems mirror communication structures of organizations",
            Domain::Lambda,
        ));
        facts.push(Fact::with_domain(
            "parkinsons_law",
            1.0,
            "ratio",
            "Work expands to fill available time",
            Domain::Lambda,
        ));
        facts.push(Fact::with_domain(
            "amara_law",
            1.0,
            "ratio",
            "We overestimate short-term impact, underestimate long-term",
            Domain::Lambda,
        ));

        // --- Tanto Merged Facts: SaaS Metrics (θ Economics) ---
        facts.push(Fact::with_domain(
            "avg_churn_rate_saas",
            5.0,
            "%/mo",
            "Typical B2B SaaS monthly churn (Recurly 2023)",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "avg_cac_payback",
            12.0,
            "months",
            "Median CAC payback period for SaaS (OpenView 2023)",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "avg_lifetime_value_saas",
            3000.0,
            "USD",
            "Median B2B SaaS LTV (Baremetrics)",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "avg_conversion_rate_saas",
            3.0,
            "%",
            "Typical B2B SaaS free-to-paid conversion rate",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "avg_gross_margin_saas",
            75.0,
            "%",
            "Median SaaS gross margin (KeyBanc 2023)",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "avg_team_productivity",
            0.6,
            "ratio",
            "Knowledge worker productive time fraction (McKinsey)",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "avg_meeting_cost",
            75.0,
            "USD/hr",
            "Avg meeting cost per attendee (Harvard Business Review)",
            Domain::Theta,
        ));
        facts.push(Fact::with_domain(
            "avg_interrupt_recovery",
            23.0,
            "min",
            "Time to refocus after interruption (UC Irvine)",
            Domain::Mu,
        ));

        // --- Climate & Environment ---
        facts.push(Fact::new(
            "global_temperature_rise",
            1.2,
            "C",
            "global temperature rise since pre-industrial",
        ));
        facts.push(Fact::new(
            "sea_level_rise",
            3.6,
            "mm/year",
            "current sea level rise rate",
        ));
        facts.push(Fact::new(
            "arctic_ice_extent",
            4.7,
            "million km2",
            "Arctic sea ice minimum extent",
        ));
        facts.push(Fact::new(
            "forest_coverage",
            31.0,
            "%",
            "global forest coverage",
        ));
        facts.push(Fact::new(
            "biodiversity_loss",
            69.0,
            "%",
            "wildlife population decline since 1970",
        ));
        facts.push(Fact::new(
            "plastic_production",
            400.0,
            "million tonnes",
            "annual global plastic production",
        ));
        facts.push(Fact::new(
            "ewaste_global",
            62.0,
            "million tonnes",
            "annual global e-waste",
        ));

        // --- Anthropology: Human Evolution ---
        facts.push(Fact::new(
            "human_chimp_divergence",
            7.0,
            "million years ago",
            "human-chimpanzee divergence",
        ));
        facts.push(Fact::new(
            "sahelanthropus",
            7.0,
            "million years ago",
            "Sahelanthropus tchadensis",
        ));
        facts.push(Fact::new(
            "orrorin",
            5.7,
            "million years ago",
            "Orrorin tugenensis",
        ));
        facts.push(Fact::new(
            "ardipithecus_kadabba",
            5.6,
            "million years ago",
            "Ardipithecus kadabba",
        ));
        facts.push(Fact::new(
            "ardipithecus_ramidus",
            4.4,
            "million years ago",
            "Ardipithecus ramidus",
        ));
        facts.push(Fact::new(
            "australopithecus_afarensis",
            3.6,
            "million years ago",
            "Australopithecus afarensis (Lucy)",
        ));
        facts.push(Fact::new(
            "homo_habilis",
            2.8,
            "million years ago",
            "Homo habilis",
        ));
        facts.push(Fact::new(
            "homo_erectus",
            1.8,
            "million years ago",
            "Homo erectus",
        ));
        facts.push(Fact::new(
            "homo_antecessor",
            1.2,
            "million years ago",
            "Homo antecessor",
        ));
        facts.push(Fact::new(
            "homo_heidelbergensis",
            0.7,
            "million years ago",
            "Homo heidelbergensis",
        ));
        facts.push(Fact::new(
            "neanderthal_appearance",
            0.4,
            "million years ago",
            "Neanderthals appear",
        ));
        facts.push(Fact::new(
            "neanderthal_extinction",
            0.04,
            "million years ago",
            "Neanderthals go extinct",
        ));
        facts.push(Fact::new(
            "homo_sapiens",
            0.3,
            "million years ago",
            "Homo sapiens anatomically modern",
        ));
        facts.push(Fact::new(
            "y_chromosomal_adam",
            0.2,
            "million years ago",
            "Y-chromosomal Adam",
        ));
        facts.push(Fact::new(
            "mitochondrial_eve",
            0.16,
            "million years ago",
            "Mitochondrial Eve",
        ));
        facts.push(Fact::new(
            "human_out_of_africa",
            0.07,
            "million years ago",
            "Modern humans leave Africa",
        ));
        facts.push(Fact::new(
            "australopithecus_brain",
            400.0,
            "cc",
            "Australopithecus brain volume",
        ));
        facts.push(Fact::new(
            "homo_habilis_brain",
            600.0,
            "cc",
            "Homo habilis brain volume",
        ));
        facts.push(Fact::new(
            "homo_erectus_brain",
            900.0,
            "cc",
            "Homo erectus brain volume",
        ));
        facts.push(Fact::new(
            "homo_sapiens_brain",
            1350.0,
            "cc",
            "Homo sapiens brain volume",
        ));
        facts.push(Fact::new(
            "neanderthal_brain",
            1500.0,
            "cc",
            "Neanderthal brain volume",
        ));
        facts.push(Fact::new(
            "first_tool_use",
            3.3,
            "million years ago",
            "Earliest stone tools (Lomekwi)",
        ));
        facts.push(Fact::new(
            "oldowan_tools",
            2.6,
            "million years ago",
            "Oldowan stone tools",
        ));
        facts.push(Fact::new(
            "acheulean_tools",
            1.76,
            "million years ago",
            "Acheulean stone tools",
        ));
        facts.push(Fact::new(
            "control_of_fire",
            1.0,
            "million years ago",
            "Earliest evidence of fire control",
        ));
        facts.push(Fact::new(
            "first_art",
            0.04,
            "million years ago",
            "Earliest known cave art",
        ));
        facts.push(Fact::new(
            "agricultural_revolution",
            0.01,
            "million years ago",
            "Neolithic Agricultural Revolution",
        ));
        facts.push(Fact::new(
            "first_cities",
            0.006,
            "million years ago",
            "First cities appear (~6000 BC)",
        ));
        facts.push(Fact::new(
            "writing_invented",
            0.005,
            "million years ago",
            "Invention of writing (~5000 BC)",
        ));

        // --- Anthropology: Primate Divergence ---
        facts.push(Fact::new(
            "primate_origin",
            85.0,
            "million years ago",
            "Primate order origin",
        ));
        facts.push(Fact::new(
            "prosimian_monkey_divergence",
            60.0,
            "million years ago",
            "Prosimian-monkey divergence",
        ));
        facts.push(Fact::new(
            "monkey_ape_divergence",
            25.0,
            "million years ago",
            "Monkey-ape divergence",
        ));
        facts.push(Fact::new(
            "great_ape_divergence",
            15.0,
            "million years ago",
            "Great ape divergence",
        ));
        facts.push(Fact::new(
            "orangutan_divergence",
            14.0,
            "million years ago",
            "Orangutan lineage diverges",
        ));
        facts.push(Fact::new(
            "gorilla_divergence",
            8.0,
            "million years ago",
            "Gorilla lineage diverges",
        ));
        facts.push(Fact::new(
            "chimpanzee_divergence",
            7.0,
            "million years ago",
            "Chimpanzee lineage diverges",
        ));
        facts.push(Fact::new(
            "human_brain_neurons",
            86.0,
            "billion",
            "Human brain neurons",
        ));
        facts.push(Fact::new(
            "human_brain_synapses",
            100.0,
            "trillion",
            "Human brain synapses",
        ));
        facts.push(Fact::new(
            "chimp_brain_neurons",
            6.0,
            "billion",
            "Chimpanzee brain neurons",
        ));

        // --- Historical Empires & Civilizations ---
        facts.push(Fact::new(
            "sumerian_civilization",
            -4500.0,
            "BC",
            "Sumerian civilization begins",
        ));
        facts.push(Fact::new(
            "egypt_old_kingdom",
            -2686.0,
            "BC",
            "Egypt Old Kingdom begins",
        ));
        facts.push(Fact::new(
            "akkadian_empire",
            -2334.0,
            "BC",
            "Akkadian Empire founded",
        ));
        facts.push(Fact::new(
            "babylonian_empire",
            -1894.0,
            "BC",
            "Old Babylonian Empire begins",
        ));
        facts.push(Fact::new(
            "hittite_empire",
            -1600.0,
            "BC",
            "Hittite Empire peak",
        ));
        facts.push(Fact::new(
            "egypt_new_kingdom",
            -1550.0,
            "BC",
            "Egypt New Kingdom begins",
        ));
        facts.push(Fact::new(
            "assyrian_empire",
            -911.0,
            "BC",
            "Neo-Assyrian Empire begins",
        ));
        facts.push(Fact::new(
            "neo_babylonian_empire",
            -626.0,
            "BC",
            "Neo-Babylonian Empire begins",
        ));
        facts.push(Fact::new(
            "persian_empire",
            -550.0,
            "BC",
            "Achaemenid Persian Empire founded",
        ));
        facts.push(Fact::new(
            "alexander_empire",
            -336.0,
            "BC",
            "Alexander the Great's empire",
        ));
        facts.push(Fact::new(
            "maurya_empire",
            -322.0,
            "BC",
            "Maurya Empire founded in India",
        ));
        facts.push(Fact::new(
            "roman_republic",
            -509.0,
            "BC",
            "Roman Republic founded",
        ));
        facts.push(Fact::new(
            "roman_empire",
            -27.0,
            "BC",
            "Roman Empire begins",
        ));
        facts.push(Fact::new(
            "han_dynasty",
            -206.0,
            "BC",
            "Han Dynasty founded in China",
        ));
        facts.push(Fact::new(
            "maurya_peak",
            -268.0,
            "BC",
            "Maurya Empire peak under Ashoka",
        ));
        facts.push(Fact::new(
            "fall_rome",
            476.0,
            "AD",
            "Fall of Western Roman Empire",
        ));
        facts.push(Fact::new(
            "byzantine_empire",
            330.0,
            "AD",
            "Byzantine Empire begins",
        ));
        facts.push(Fact::new(
            "islamic_caliphate",
            632.0,
            "AD",
            "Rashidun Caliphate begins",
        ));
        facts.push(Fact::new(
            "tang_dynasty",
            618.0,
            "AD",
            "Tang Dynasty in China",
        ));
        facts.push(Fact::new(
            "carolingian_empire",
            800.0,
            "AD",
            "Charlemagne's Carolingian Empire",
        ));
        facts.push(Fact::new(
            "holy_roman_empire",
            962.0,
            "AD",
            "Holy Roman Empire begins",
        ));
        facts.push(Fact::new(
            "song_dynasty",
            960.0,
            "AD",
            "Song Dynasty in China",
        ));
        facts.push(Fact::new(
            "mongol_empire",
            1206.0,
            "AD",
            "Mongol Empire founded",
        ));
        facts.push(Fact::new(
            "ottoman_empire",
            1299.0,
            "AD",
            "Ottoman Empire begins",
        ));
        facts.push(Fact::new(
            "ming_dynasty",
            1368.0,
            "AD",
            "Ming Dynasty in China",
        ));
        facts.push(Fact::new(
            "timurid_empire",
            1370.0,
            "AD",
            "Timurid Empire begins",
        ));
        facts.push(Fact::new(
            "portuguese_empire",
            1415.0,
            "AD",
            "Portuguese Empire begins",
        ));
        facts.push(Fact::new(
            "spanish_empire",
            1492.0,
            "AD",
            "Spanish Empire begins",
        ));
        facts.push(Fact::new(
            "mughal_empire",
            1526.0,
            "AD",
            "Mughal Empire founded",
        ));
        facts.push(Fact::new(
            "british_empire",
            1583.0,
            "AD",
            "British Empire begins",
        ));
        facts.push(Fact::new(
            "french_colonial",
            1600.0,
            "AD",
            "French colonial empire begins",
        ));
        facts.push(Fact::new(
            "dutch_empire",
            1602.0,
            "AD",
            "Dutch East India Company",
        ));
        facts.push(Fact::new(
            "russian_empire",
            1721.0,
            "AD",
            "Russian Empire proclaimed",
        ));
        facts.push(Fact::new(
            "qing_dynasty",
            1644.0,
            "AD",
            "Qing Dynasty in China",
        ));
        facts.push(Fact::new(
            "napoleon_empire",
            1804.0,
            "AD",
            "Napoleon's First French Empire",
        ));
        facts.push(Fact::new(
            "german_empire",
            1871.0,
            "AD",
            "German Empire proclaimed",
        ));
        facts.push(Fact::new(
            "japanese_empire",
            1868.0,
            "AD",
            "Empire of Japan begins",
        ));
        facts.push(Fact::new(
            "soviet_union",
            1922.0,
            "AD",
            "Soviet Union founded",
        ));
        facts.push(Fact::new(
            "third_reich",
            1933.0,
            "AD",
            "Nazi Germany (Third Reich)",
        ));
        facts.push(Fact::new(
            "roman_empire_peak",
            117.0,
            "AD",
            "Roman Empire peak territory",
        ));
        facts.push(Fact::new(
            "mongol_empire_peak",
            1270.0,
            "AD",
            "Mongol Empire peak territory",
        ));
        facts.push(Fact::new(
            "british_empire_peak",
            1920.0,
            "AD",
            "British Empire peak territory",
        ));

        // --- Historical Population Estimates ---
        facts.push(Fact::new(
            "world_population_10000bc",
            5.0,
            "million",
            "world population ~10000 BC",
        ));
        facts.push(Fact::new(
            "world_population_1ad",
            200.0,
            "million",
            "world population ~1 AD",
        ));
        facts.push(Fact::new(
            "world_population_1000ad",
            310.0,
            "million",
            "world population ~1000 AD",
        ));
        facts.push(Fact::new(
            "world_population_1250ad",
            400.0,
            "million",
            "world population ~1250 AD",
        ));
        facts.push(Fact::new(
            "world_population_1500ad",
            460.0,
            "million",
            "world population ~1500 AD",
        ));
        facts.push(Fact::new(
            "world_population_1750ad",
            790.0,
            "million",
            "world population ~1750 AD",
        ));
        facts.push(Fact::new(
            "world_population_1850ad",
            1260.0,
            "million",
            "world population ~1850 AD",
        ));
        facts.push(Fact::new(
            "world_population_1900ad",
            1650.0,
            "million",
            "world population ~1900 AD",
        ));
        facts.push(Fact::new(
            "world_population_1950ad",
            2536.0,
            "million",
            "world population ~1950 AD",
        ));
        facts.push(Fact::new(
            "world_population_2000ad",
            6143.0,
            "million",
            "world population ~2000 AD",
        ));
        facts.push(Fact::new(
            "world_population_2024ad",
            8100.0,
            "million",
            "world population ~2024 AD",
        ));

        // --- Historical Empires by Peak Population ---
        facts.push(Fact::new(
            "roman_empire_population",
            60.0,
            "million",
            "Roman Empire peak population",
        ));
        facts.push(Fact::new(
            "han_dynasty_population",
            60.0,
            "million",
            "Han Dynasty peak population",
        ));
        facts.push(Fact::new(
            "mongol_empire_population",
            100.0,
            "million",
            "Mongol Empire peak population",
        ));
        facts.push(Fact::new(
            "mughal_empire_population",
            150.0,
            "million",
            "Mughal Empire peak population",
        ));
        facts.push(Fact::new(
            "qing_dynasty_population",
            432.0,
            "million",
            "Qing Dynasty peak population",
        ));
        facts.push(Fact::new(
            "british_empire_population",
            458.0,
            "million",
            "British Empire peak population",
        ));
        facts.push(Fact::new(
            "soviet_union_population",
            293.0,
            "million",
            "Soviet Union peak population",
        ));
        facts.push(Fact::new(
            "maurya_empire_population",
            50.0,
            "million",
            "Maurya Empire peak population",
        ));

        // --- Historical Empires by Peak Area ---
        facts.push(Fact::new(
            "british_empire_area",
            35.5,
            "million km2",
            "British Empire peak area",
        ));
        facts.push(Fact::new(
            "mongol_empire_area",
            24.0,
            "million km2",
            "Mongol Empire peak area",
        ));
        facts.push(Fact::new(
            "roman_empire_area",
            5.0,
            "million km2",
            "Roman Empire peak area",
        ));
        facts.push(Fact::new(
            "qing_dynasty_area",
            14.7,
            "million km2",
            "Qing Dynasty peak area",
        ));
        facts.push(Fact::new(
            "spanish_empire_area",
            20.0,
            "million km2",
            "Spanish Empire peak area",
        ));
        facts.push(Fact::new(
            "russian_empire_area",
            22.8,
            "million km2",
            "Russian Empire peak area",
        ));
        facts.push(Fact::new(
            "french_colonial_area",
            11.5,
            "million km2",
            "French colonial empire peak area",
        ));
        facts.push(Fact::new(
            "austrian_hungarian_area",
            0.68,
            "million km2",
            "Austro-Hungarian Empire area",
        ));
        facts.push(Fact::new(
            "ottoman_empire_area",
            5.2,
            "million km2",
            "Ottoman Empire peak area",
        ));
        facts.push(Fact::new(
            "persian_empire_area",
            5.5,
            "million km2",
            "Achaemenid Persian Empire area",
        ));

        // --- Cultural Universals ---
        facts.push(Fact::new(
            "language_universal",
            1.0,
            "",
            "all cultures have language (universal)",
        ));
        facts.push(Fact::new(
            "music_universal",
            1.0,
            "",
            "all cultures have music (universal)",
        ));
        facts.push(Fact::new(
            "dance_universal",
            1.0,
            "",
            "all cultures have dance (universal)",
        ));
        facts.push(Fact::new(
            "religion_universal",
            1.0,
            "",
            "all cultures have religion/belief (universal)",
        ));
        facts.push(Fact::new(
            "kinship_universal",
            1.0,
            "",
            "all cultures have kinship systems (universal)",
        ));
        facts.push(Fact::new(
            "food_universal",
            1.0,
            "",
            "all cultures have food customs (universal)",
        ));
        facts.push(Fact::new(
            "jokes_universal",
            1.0,
            "",
            "all cultures have humor/jokes (universal)",
        ));
        facts.push(Fact::new(
            "trade_universal",
            1.0,
            "",
            "all cultures have trade/exchange (universal)",
        ));
        facts.push(Fact::new(
            "body_modification_universal",
            1.0,
            "",
            "all cultures have body modification (universal)",
        ));
        facts.push(Fact::new(
            "cooperation_universal",
            1.0,
            "",
            "all cultures have cooperation norms (universal)",
        ));

        // --- Key Historical Events ---
        facts.push(Fact::new(
            "neolithic_revolution",
            -10000.0,
            "BC",
            "Neolithic Revolution begins",
        ));
        facts.push(Fact::new(
            "first_wheel",
            -3500.0,
            "BC",
            "Invention of the wheel",
        ));
        facts.push(Fact::new("bronze_age", -3300.0, "BC", "Bronze Age begins"));
        facts.push(Fact::new("iron_age", -1200.0, "BC", "Iron Age begins"));
        facts.push(Fact::new(
            "axial_age",
            -800.0,
            "BC",
            "Axial Age (philosophical revolution)",
        ));
        facts.push(Fact::new(
            "birth_christianity",
            0.0,
            "AD",
            "Birth of Christianity",
        ));
        facts.push(Fact::new(
            "birth_islam",
            622.0,
            "AD",
            "Birth of Islam (Hijra)",
        ));
        facts.push(Fact::new(
            "black_death",
            1347.0,
            "AD",
            "Black Death reaches Europe",
        ));
        facts.push(Fact::new("renaissance", 1400.0, "AD", "Renaissance begins"));
        facts.push(Fact::new(
            "reformation",
            1517.0,
            "AD",
            "Protestant Reformation",
        ));
        facts.push(Fact::new(
            "scientific_revolution",
            1543.0,
            "AD",
            "Scientific Revolution begins",
        ));
        facts.push(Fact::new(
            "enlightenment",
            1685.0,
            "AD",
            "Enlightenment begins",
        ));
        facts.push(Fact::new(
            "industrial_revolution",
            1760.0,
            "AD",
            "Industrial Revolution begins",
        ));
        facts.push(Fact::new(
            "french_revolution_event",
            1789.0,
            "AD",
            "French Revolution",
        ));
        facts.push(Fact::new(
            "american_revolution",
            1775.0,
            "AD",
            "American Revolution",
        ));
        facts.push(Fact::new("world_war_1", 1914.0, "AD", "World War I"));
        facts.push(Fact::new("world_war_2", 1939.0, "AD", "World War II"));
        facts.push(Fact::new("cold_war", 1947.0, "AD", "Cold War begins"));
        facts.push(Fact::new(
            "information_age",
            1970.0,
            "AD",
            "Information Age begins",
        ));
        facts.push(Fact::new(
            "internet_era",
            1991.0,
            "AD",
            "Internet era begins",
        ));

        let index: HashMap<String, usize> = facts
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        // Build domain index
        let mut domain_index: HashMap<Domain, Vec<usize>> = HashMap::new();
        for (i, fact) in facts.iter().enumerate() {
            domain_index.entry(fact.domain).or_default().push(i);
        }

        KnowledgeBase {
            facts,
            index,
            domain_index,
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Fact> {
        self.index.get(name).and_then(|&idx| self.facts.get(idx))
    }

    pub fn lookup_value(&self, name: &str) -> Option<f64> {
        self.lookup(name).map(|f| f.value)
    }

    pub fn add_fact(&mut self, fact: Fact) {
        let idx = self.facts.len();
        let domain = fact.domain;
        self.index.insert(fact.name.clone(), idx);
        self.domain_index.entry(domain).or_default().push(idx);
        self.facts.push(fact);
    }

    pub fn remove_fact(&mut self, name: &str) -> bool {
        let len_before = self.facts.len();
        self.facts.retain(|f| f.name != name);
        let removed = self.facts.len() < len_before;
        if removed {
            self.rebuild_index();
        }
        removed
    }

    fn rebuild_index(&mut self) {
        self.index = self
            .facts
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        // Rebuild domain index
        self.domain_index.clear();
        for (i, fact) in self.facts.iter().enumerate() {
            self.domain_index.entry(fact.domain).or_default().push(i);
        }
    }

    pub fn list_facts(&self) -> &[Fact] {
        &self.facts
    }

    pub fn count(&self) -> usize {
        self.facts.len()
    }

    /// Count facts in a specific domain
    pub fn count_domain(&self, domain: Domain) -> usize {
        self.domain_index.get(&domain).map_or(0, |v| v.len())
    }

    /// Get all facts in a specific domain
    pub fn facts_by_domain(&self, domain: Domain) -> Vec<&Fact> {
        self.domain_index
            .get(&domain)
            .map(|indices| indices.iter().filter_map(|&i| self.facts.get(i)).collect())
            .unwrap_or_default()
    }

    /// Get all facts with secondary domain tag
    pub fn search(&self, query: &str) -> Vec<&Fact> {
        let lower_query = query.to_lowercase();
        self.facts
            .iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&lower_query)
                    || f.source.to_lowercase().contains(&lower_query)
                    || f.unit.to_lowercase().contains(&lower_query)
            })
            .collect()
    }

    /// Get domain statistics
    pub fn domain_stats(&self) -> Vec<(Domain, usize)> {
        Domain::all()
            .into_iter()
            .map(|d| (d, self.count_domain(d)))
            .collect()
    }

    /// Get all facts as a list with domain info
    pub fn list_facts_with_domain(&self) -> Vec<(&Fact, &str)> {
        self.facts
            .iter()
            .map(|f| (f, f.domain.description()))
            .collect()
    }

    /// Get facts by domain name string
    pub fn facts_by_domain_name(&self, domain_name: &str) -> Vec<&Fact> {
        if let Some(domain) = Domain::from_name(domain_name) {
            self.facts_by_domain(domain)
        } else {
            Vec::new()
        }
    }

    /// Get domain for a fact by name
    pub fn fact_domain(&self, name: &str) -> Option<Domain> {
        self.lookup(name).map(|f| f.domain)
    }

    /// Get all facts with their domains as strings
    pub fn facts_with_domains(&self) -> Vec<(String, f64, String, String)> {
        self.facts
            .iter()
            .map(|f| {
                (
                    f.name.clone(),
                    f.value,
                    f.unit.clone(),
                    f.domain.description().to_string(),
                )
            })
            .collect()
    }

    /// Search facts by domain + keyword, sorted by recency (newest first)
    pub fn search_by_domain(&self, query: &str, domain: Domain) -> Vec<&Fact> {
        let lower_query = query.to_lowercase();
        let mut results: Vec<&Fact> = self
            .facts
            .iter()
            .filter(|f| {
                f.domain == domain
                    && (f.name.to_lowercase().contains(&lower_query)
                        || f.source.to_lowercase().contains(&lower_query)
                        || f.unit.to_lowercase().contains(&lower_query))
            })
            .collect();
        results.sort_by_key(|f| std::cmp::Reverse(f.created_at));
        results
    }

    /// Get top N most relevant facts for a given context (domain + query)
    /// Relevance score = keyword_match_weight + recency_weight + confidence_weight
    pub fn context_window(
        &self,
        query: &str,
        domain: Domain,
        max_results: usize,
    ) -> Vec<(&Fact, f64)> {
        let lower_query = query.to_lowercase();
        let now = now_epoch();
        let mut scored: Vec<(&Fact, f64)> = self
            .facts
            .iter()
            .filter_map(|f| {
                let mut score = 0.0_f64;
                // Domain match bonus
                if f.domain == domain {
                    score += 0.3;
                }
                // Keyword match
                let name_match = f.name.to_lowercase().contains(&lower_query);
                let source_match = f.source.to_lowercase().contains(&lower_query);
                if name_match {
                    score += 0.4;
                }
                if source_match {
                    score += 0.2;
                }
                // Recency (facts newer than 30 days get bonus)
                if now >= f.created_at && (now - f.created_at) < 2592000 {
                    score += 0.1;
                }
                // Confidence
                score *= f.confidence;
                if score > 0.0 { Some((f, score)) } else { None }
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(max_results);
        scored
    }

    /// Get newest facts added within time window (seconds)
    pub fn recent_facts(&self, within_secs: u64) -> Vec<&Fact> {
        let now = now_epoch();
        self.facts
            .iter()
            .filter(|f| now >= f.created_at && (now - f.created_at) < within_secs)
            .collect()
    }

    /// Get facts with confidence above threshold
    pub fn facts_by_confidence(&self, min_confidence: f64) -> Vec<&Fact> {
        self.facts
            .iter()
            .filter(|f| f.confidence >= min_confidence)
            .collect()
    }

    /// Invalidate a dynamic fact (set confidence to 0)
    pub fn invalidate_fact(&mut self, name: &str) -> bool {
        if let Some(fact) = self.facts.iter_mut().find(|f| f.name == name) {
            fact.confidence = 0.0;
            true
        } else {
            false
        }
    }

    /// Update an existing fact's value (keeps original created_at, updates confidence)
    pub fn update_fact(&mut self, name: &str, new_value: f64, new_confidence: f64) -> bool {
        if let Some(fact) = self.facts.iter_mut().find(|f| f.name == name) {
            fact.value = new_value;
            fact.confidence = new_confidence.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }
}
