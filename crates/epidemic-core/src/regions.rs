/// A game region — groups of countries or standalone large countries.
#[derive(Debug, Clone)]
pub struct Region {
    pub id: u16,           // matches SVG lookup table
    pub code: String,      // primary ISO code (for display)
    pub name: String,      // display name
    pub population: u64,   // 2026 estimate
    pub infected: u64,
    pub dead: u64,
    pub borders_open: bool,
    pub cure_progress: f32, // 0.0 to 100.0
    pub fallen: bool,       // true when everyone is dead
    pub svg_codes: Vec<String>, // all SVG country codes in this region
}

impl Region {
    pub fn new(id: u16, code: &str, name: &str, population: u64, svg_codes: &[&str]) -> Self {
        Self {
            id,
            code: code.to_string(),
            name: name.to_string(),
            population,
            infected: 0,
            dead: 0,
            borders_open: true,
            cure_progress: 0.0,
            fallen: false,
            svg_codes: svg_codes.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn healthy(&self) -> u64 {
        self.population.saturating_sub(self.infected + self.dead)
    }

    pub fn infection_pct(&self) -> f32 {
        if self.population == 0 {
            return 0.0;
        }
        self.infected as f32 / self.population as f32
    }

    pub fn death_pct(&self) -> f32 {
        if self.population == 0 {
            return 0.0;
        }
        self.dead as f32 / self.population as f32
    }
}

/// Build the default region definitions.
/// ~60-80 regions: large countries standalone, medium/small grouped.
pub fn build_regions() -> Vec<Region> {
    vec![
        // North America
        Region::new(1, "US", "United States", 341_800_000, &["US"]),
        Region::new(2, "CA", "Canada", 41_000_000, &["CA"]),
        Region::new(3, "MX", "Mexico", 130_000_000, &["MX"]),
        // Central America + Caribbean
        Region::new(4, "CAM", "Central America", 55_000_000, &[
            "GT", "BZ", "HN", "SV", "NI", "CR", "PA",
            "CU", "JM", "HT", "DO", "TT", "BB", "GD",
            "LC", "VC", "AG", "KN", "DM", "BS",
            "PR", "VI", "AI", "AW", "CW", "SX", "MF", "BL", "PM",
        ]),
        // South America
        Region::new(5, "BR", "Brazil", 216_000_000, &["BR"]),
        Region::new(6, "AR", "Argentina", 47_000_000, &["AR"]),
        Region::new(7, "CO", "Colombia", 52_000_000, &["CO"]),
        Region::new(8, "PE", "Peru", 34_000_000, &["PE"]),
        Region::new(9, "VE", "Venezuela", 28_000_000, &["VE"]),
        Region::new(10, "SA", "South America Rest", 45_000_000, &[
            "CL", "EC", "BO", "PY", "UY", "GY", "SR", "GF",
        ]),
        // Western Europe
        Region::new(11, "GB", "United Kingdom", 69_000_000, &["GB"]),
        Region::new(12, "FR", "France", 68_000_000, &["FR"]),
        Region::new(13, "DE", "Germany", 84_000_000, &["DE"]),
        Region::new(14, "ES", "Spain", 48_000_000, &["ES"]),
        Region::new(15, "PT", "Portugal", 10_400_000, &["PT"]),
        Region::new(16, "IT", "Italy", 59_000_000, &["IT"]),
        Region::new(17, "WE", "Western Europe", 75_000_000, &[
            "NL", "BE", "LU", "CH", "AT", "IE",
        ]),
        // Northern Europe
        Region::new(18, "NE", "Northern Europe", 28_000_000, &[
            "SE", "NO", "DK", "FI", "IS",
        ]),
        // Eastern Europe
        Region::new(19, "PL", "Poland", 38_000_000, &["PL"]),
        Region::new(20, "UA", "Ukraine", 37_000_000, &["UA"]),
        Region::new(21, "EE", "Eastern Europe", 85_000_000, &[
            "CZ", "SK", "HU", "RO", "BG", "HR", "RS", "BA",
            "ME", "MK", "AL", "XK", "SI", "EE", "LV", "LT", "MD", "BY", "GE", "AM", "AZ",
        ]),
        // Russia
        Region::new(22, "RU", "Russia", 144_000_000, &["RU"]),
        // Middle East
        Region::new(23, "TR", "Turkey", 86_000_000, &["TR"]),
        Region::new(24, "SA2", "Saudi Arabia", 37_000_000, &["SA"]),
        Region::new(25, "IR", "Iran", 88_000_000, &["IR"]),
        Region::new(26, "IQ", "Iraq", 43_000_000, &["IQ"]),
        Region::new(27, "ME", "Middle East Rest", 65_000_000, &[
            "AE", "IL", "JO", "LB", "SY", "YE", "OM", "QA", "BH", "KW", "PS",
        ]),
        // North Africa
        Region::new(28, "EG", "Egypt", 106_000_000, &["EG"]),
        Region::new(29, "DZ", "Algeria", 46_000_000, &["DZ"]),
        Region::new(30, "MA", "Morocco", 37_500_000, &["MA"]),
        Region::new(31, "NA", "North Africa Rest", 45_000_000, &[
            "TN", "LY", "SD", "SS", "EH",
        ]),
        // West Africa
        Region::new(32, "NG", "Nigeria", 224_000_000, &["NG"]),
        Region::new(33, "GH", "Ghana", 34_000_000, &["GH"]),
        Region::new(34, "WA", "West Africa Rest", 180_000_000, &[
            "SN", "ML", "BF", "NE", "CI", "GN", "SL", "LR", "BJ", "TG",
            "MR", "GM", "GW", "GN", "CV",
        ]),
        // East Africa
        Region::new(35, "ET", "Ethiopia", 126_000_000, &["ET"]),
        Region::new(36, "KE", "Kenya", 56_000_000, &["KE"]),
        Region::new(37, "TZ", "Tanzania", 65_000_000, &["TZ"]),
        Region::new(38, "EA", "East Africa Rest", 140_000_000, &[
            "UG", "RW", "BI", "DJ", "ER", "SO", "MG", "MZ", "MW", "ZM", "ZW",
        ]),
        // Central Africa
        Region::new(39, "CD", "DR Congo", 102_000_000, &["CD"]),
        Region::new(40, "CF", "Central Africa Rest", 70_000_000, &[
            "CM", "CG", "GA", "GQ", "TD", "CF", "AO", "ST",
        ]),
        // Southern Africa
        Region::new(41, "ZA", "South Africa", 62_000_000, &["ZA"]),
        Region::new(42, "SA3", "Southern Africa Rest", 30_000_000, &[
            "NA", "BW", "SZ", "LS",
        ]),
        // Central Asia
        Region::new(43, "KZ", "Kazakhstan", 20_000_000, &["KZ"]),
        Region::new(44, "CA2", "Central Asia Rest", 60_000_000, &[
            "UZ", "TM", "KG", "TJ", "AF", "MN",
        ]),
        // South Asia
        Region::new(45, "IN", "India", 1_450_000_000, &["IN"]),
        Region::new(46, "PK", "Pakistan", 240_000_000, &["PK"]),
        Region::new(47, "BD", "Bangladesh", 175_000_000, &["BD"]),
        Region::new(48, "SA4", "South Asia Rest", 55_000_000, &[
            "NP", "LK", "BT", "MV", "AF",
        ]),
        // Southeast Asia
        Region::new(49, "ID", "Indonesia", 280_000_000, &["ID"]),
        Region::new(50, "TH", "Thailand", 72_000_000, &["TH"]),
        Region::new(51, "VN", "Vietnam", 100_000_000, &["VN"]),
        Region::new(52, "PH", "Philippines", 117_000_000, &["PH"]),
        Region::new(53, "MM", "Myanmar", 55_000_000, &["MM"]),
        Region::new(54, "MY", "Malaysia", 34_000_000, &["MY"]),
        Region::new(55, "SEA", "Southeast Asia Rest", 60_000_000, &[
            "KH", "LA", "BN", "TL", "SG", "BN",
        ]),
        // East Asia
        Region::new(56, "CN", "China", 1_425_000_000, &["CN"]),
        Region::new(57, "JP", "Japan", 124_000_000, &["JP"]),
        Region::new(58, "KR", "South Korea", 52_000_000, &["KR"]),
        Region::new(59, "KP", "North Korea", 26_000_000, &["KP"]),
        Region::new(60, "TW", "Taiwan", 24_000_000, &["TW"]),
        // Oceania
        Region::new(61, "AU", "Australia", 27_000_000, &["AU"]),
        Region::new(62, "NZ", "New Zealand", 5_200_000, &["NZ"]),
        Region::new(63, "OC", "Oceania Rest", 15_000_000, &[
            "PG", "FJ", "SB", "VU", "WS", "TO", "KI", "MH",
            "FM", "PW", "TV", "NR", "NC", "PF", "GU",
        ]),
        // Greenland
        Region::new(64, "GL", "Greenland", 57_000, &["GL"]),
    ]
}

/// Map SVG country codes to region IDs.
/// This is used to build the lookup table from the rasterized SVG.
pub fn svg_code_to_region_map(regions: &[Region]) -> std::collections::HashMap<String, u16> {
    let mut map = std::collections::HashMap::new();
    for region in regions {
        for code in &region.svg_codes {
            map.insert(code.clone(), region.id);
        }
    }
    map
}
