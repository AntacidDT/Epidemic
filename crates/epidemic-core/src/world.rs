use crate::disease::{Disease, PathogenType, Upgrade, all_upgrades};
use crate::regions::{self, Region, Climate, Density, GovernmentType};
use crate::svg_parser;

// ─────────────────────────────────────────────────────────────
// Core Types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    TitleScreen,
    PathogenSelect,
    DifficultySelect,
    SelectOrigin,
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Casual,
    Normal,
    Brutal,
    MegaBrutal,
}

impl Difficulty {
    pub fn name(&self) -> &str {
        match self {
            Self::Casual => "Casual",
            Self::Normal => "Normal",
            Self::Brutal => "Brutal",
            Self::MegaBrutal => "Mega Brutal",
        }
    }
    pub fn cure_speed_mult(&self) -> f32 {
        match self { Self::Casual => 0.5, Self::Normal => 1.0, Self::Brutal => 1.5, Self::MegaBrutal => 2.0 }
    }
    pub fn border_close_mult(&self) -> f32 {
        match self { Self::Casual => 0.5, Self::Normal => 1.0, Self::Brutal => 1.5, Self::MegaBrutal => 2.0 }
    }
}

// ─────────────────────────────────────────────────────────────
// Transport Entities (Physical Planes & Ships)
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TransportEntity {
    pub id: u32,
    pub origin: u16,          // region ID
    pub destination: u16,     // region ID
    pub transport_type: TransportType,
    pub progress: f32,        // 0.0 to 1.0
    pub infected_passengers: u64,
    pub total_passengers: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportType {
    Flight,
    CargoShip,
}

impl TransportType {
    pub fn speed(&self) -> f32 {
        match self { Self::Flight => 0.02, Self::CargoShip => 0.005 }
    }
}

// ─────────────────────────────────────────────────────────────
// Cure Phases
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurePhase {
    Inactive,
    Research,        // Phase 1: Finding genetic sequence
    Trials,          // Phase 2: Human trials
    Manufacturing,   // Phase 3: Producing vaccines
    Distribution,    // Phase 4: Delivering to regions
    Complete,
}

impl CurePhase {
    pub fn name(&self) -> &str {
        match self {
            Self::Inactive => "Inactive",
            Self::Research => "Research",
            Self::Trials => "Trials",
            Self::Manufacturing => "Manufacturing",
            Self::Distribution => "Distribution",
            Self::Complete => "Complete",
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GameEvent {
    pub tick: u64,
    pub message: String,
    pub event_type: EventType,
}

#[derive(Debug, Clone)]
pub enum EventType {
    NewCountry(String),
    BorderClosed(String),
    CurePhaseChange(CurePhase),
    HealthcareCollapse(String),
    SupplyChainBreak(String),
    SportsEvent(String),
    ResearchBoost,
    MisinformationWave(String),
    FlightInfected(String, String),
    ShipInfected(String, String),
}

// ─────────────────────────────────────────────────────────────
// DNA Bubbles
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DnaBubble {
    pub x: f32,
    pub y: f32,
    pub value: u32,
    pub tick_spawned: u64,
    pub collected: bool,
}

// ─────────────────────────────────────────────────────────────
// Tactical Abilities
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TacticalAbility {
    pub id: &'static str,
    pub name: &'static str,
    pub cost: u32,
    pub cooldown_ticks: u64,
    pub last_used: u64,
    pub description: &'static str,
}

pub fn all_tactical_abilities() -> Vec<TacticalAbility> {
    vec![
        TacticalAbility { id: "spore_strike", name: "Spore Strike", cost: 15, cooldown_ticks: 300, last_used: 0,
            description: "Launch spores to infect a targeted region" },
        TacticalAbility { id: "symptom_cloak", name: "Symptom Cloak", cost: 10, cooldown_ticks: 500, last_used: 0,
            description: "Suppress visible symptoms in a region for 100 ticks" },
        TacticalAbility { id: "infectious_surge", name: "Infectious Surge", cost: 12, cooldown_ticks: 400, last_used: 0,
            description: "Trigger mass gathering in a region, spiking infections" },
    ]
}

// ─────────────────────────────────────────────────────────────
// Symptom Synergies
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Synergy {
    pub id: &'static str,
    pub name: &'static str,
    pub requires: Vec<&'static str>,
    pub bonus_infectivity: f32,
    pub bonus_severity: f32,
    pub bonus_lethality: f32,
    pub special_effect: &'static str,
    pub unlocked: bool,
}

pub fn all_synergies() -> Vec<Synergy> {
    vec![
        Synergy {
            id: "aerosolized_dispersal", name: "Aerosolized Dispersal",
            requires: vec!["coughing", "diarrhea"],
            bonus_infectivity: 5.0, bonus_severity: 0.0, bonus_lethality: 0.0,
            special_effect: "+20% infection in humid (Tropical) climates",
            unlocked: false,
        },
        Synergy {
            id: "mass_hysteria", name: "Mass Hysteria",
            requires: vec!["insomnia", "paranoia", "seizures"],
            bonus_infectivity: 0.0, bonus_severity: 5.0, bonus_lethality: 2.0,
            special_effect: "Causes healthcare collapse in affected region",
            unlocked: false,
        },
        Synergy {
            id: "hemorrhagic_pneumonia", name: "Hemorrhagic Pneumonia",
            requires: vec!["pneumonia", "hemorrhagic_shock"],
            bonus_infectivity: 3.0, bonus_severity: 4.0, bonus_lethality: 8.0,
            special_effect: "Extreme lethality + airborne spread",
            unlocked: false,
        },
        Synergy {
            id: "necrotic_fasciitis", name: "Necrotic Fasciitis",
            requires: vec!["necrosis", "skin_lesions"],
            bonus_infectivity: 2.0, bonus_severity: 6.0, bonus_lethality: 6.0,
            special_effect: "Flesh-eating — extreme fear and panic",
            unlocked: false,
        },
        Synergy {
            id: "total_systemic_collapse", name: "Total Systemic Collapse",
            requires: vec!["total_organ_failure", "immune_suppression", "coma"],
            bonus_infectivity: 0.0, bonus_severity: 10.0, bonus_lethality: 15.0,
            special_effect: "Guaranteed death — no recovery possible",
            unlocked: false,
        },
    ]
}

// ─────────────────────────────────────────────────────────────
// Global Season
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Season {
    Spring,  // Northern temperate, Southern temperate
    Summer,  // Northern hot, Southern cold
    Autumn,  // Northern temperate, Southern temperate
    Winter,  // Northern cold, Southern hot
}

impl Season {
    pub fn from_tick(tick: u64) -> Self {
        // Each season lasts 500 ticks (2000 tick year)
        match (tick / 500) % 4 {
            0 => Self::Spring,
            1 => Self::Summer,
            2 => Self::Autumn,
            3 => Self::Winter,
            _ => Self::Spring,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Spring => "Spring", Self::Summer => "Summer",
            Self::Autumn => "Autumn", Self::Winter => "Winter",
        }
    }
}

// ─────────────────────────────────────────────────────────────
// World
// ─────────────────────────────────────────────────────────────

pub struct World {
    pub regions: Vec<Region>,
    pub svg_lookup: Vec<u16>,
    pub lookup_w: usize,
    pub lookup_h: usize,
    pub tick: u64,
    pub game_speed: u32,
    pub dna_points: u32,
    pub total_infected: u64,
    pub total_dead: u64,
    pub total_healthy: u64,
    pub news: Vec<String>,
    pub phase: GamePhase,
    pub selected_region: Option<u16>,
    pub disease: Disease,
    pub upgrades: Vec<Upgrade>,
    pub events: Vec<GameEvent>,
    pub dna_bubbles: Vec<DnaBubble>,
    pub difficulty: Difficulty,
    pub disease_name: String,

    // Transport
    pub transports: Vec<TransportEntity>,
    pub next_transport_id: u32,

    // Multi-stage cure
    pub cure_phase: CurePhase,
    pub cure_research_progress: f32,   // 0-100 within current phase
    pub cure_trials_progress: f32,     // 0-100
    pub cure_manufacturing_progress: f32, // 0-100
    pub cure_distribution_progress: f32, // 0-100
    pub cure_overall: f32,             // 0-100 (weighted sum)

    // Global supply chain
    pub global_manufacturing: f32,     // 0-1.0 aggregate
    pub global_agriculture: f32,       // 0-1.0 aggregate

    // Season
    pub season: Season,

    // Tactical
    pub tactical_abilities: Vec<TacticalAbility>,
    pub symp_cloaked_regions: Vec<(u16, u64)>, // (region_id, expires_tick)

    // Synergies
    pub synergies: Vec<Synergy>,

    // Global stats
    pub global_panic: f32,             // 0-1.0 average
}

impl World {
    pub fn new(svg_content: &str) -> Self {
        let regions = regions::build_regions();
        let code_to_region = regions::svg_code_to_region_map(&regions);
        let lookup_w = 800;
        let lookup_h = 400;
        let (raw_lookup, svg_countries) = svg_parser::rasterize_svg_to_lookup(svg_content, lookup_w, lookup_h);

        let mut svg_code_list: Vec<String> = svg_countries.iter().map(|c| c.code.clone()).collect();
        svg_code_list.insert(0, "".to_string());

        let mut lookup = vec![0u16; lookup_w * lookup_h];
        for (i, &raw_id) in raw_lookup.iter().enumerate() {
            if raw_id == 0 { lookup[i] = 0; }
            else if let Some(code) = svg_code_list.get(raw_id as usize) {
                lookup[i] = code_to_region.get(code).copied().unwrap_or(0);
            }
        }

        let total_pop: u64 = regions.iter().map(|r| r.population).sum();

        Self {
            regions, svg_lookup: lookup, lookup_w, lookup_h,
            tick: 0, game_speed: 1, dna_points: 0,
            total_infected: 0, total_dead: 0, total_healthy: total_pop,
            news: Vec::new(), phase: GamePhase::SelectOrigin, selected_region: None,
            disease: Disease::new("Unknown", PathogenType::Bacteria),
            upgrades: all_upgrades(), events: Vec::new(), dna_bubbles: Vec::new(),
            difficulty: Difficulty::Normal, disease_name: "Epidemic".to_string(),
            transports: Vec::new(), next_transport_id: 1,
            cure_phase: CurePhase::Inactive, cure_research_progress: 0.0,
            cure_trials_progress: 0.0, cure_manufacturing_progress: 0.0,
            cure_distribution_progress: 0.0, cure_overall: 0.0,
            global_manufacturing: 0.7, global_agriculture: 0.7,
            season: Season::Spring,
            tactical_abilities: all_tactical_abilities(),
            symp_cloaked_regions: Vec::new(),
            synergies: all_synergies(),
            global_panic: 0.0,
        }
    }

    pub fn init_disease(&mut self, name: &str, pathogen_type: PathogenType) {
        self.disease = Disease::new(name, pathogen_type);
        self.disease_name = name.to_string();
    }

    pub fn start_outbreak(&mut self, region_id: u16) {
        if let Some(region) = self.regions.iter_mut().find(|r| r.id == region_id) {
            region.infected = 1;
            let name = region.name.clone();
            self.news.push(format!("Outbreak detected in {name}!"));
            self.phase = GamePhase::Playing;
            self.selected_region = Some(region_id);
        }
    }

    pub fn region_at_pixel(&self, px: usize, py: usize) -> Option<&Region> {
        if px >= self.lookup_w || py >= self.lookup_h { return None; }
        let id = self.svg_lookup[py * self.lookup_w + px];
        if id == 0 { return None; }
        self.regions.iter().find(|r| r.id == id)
    }

    // ─────────────────────────────────────────────────────────
    // Main simulation tick
    // ─────────────────────────────────────────────────────────

    pub fn advance(&mut self) {
        if self.phase != GamePhase::Playing { return; }

        self.tick += 1;
        self.season = Season::from_tick(self.tick);

        let infectivity = self.disease.effective_infectivity();
        let severity = self.disease.effective_severity();
        let lethality = self.disease.effective_lethality();

        // DNA passive income
        if self.total_infected > 0 && self.tick % 50 == 0 {
            self.dna_points += 1 + (self.total_infected / 10_000_000).min(5) as u32;
        }

        // Spawn DNA bubbles
        if self.total_infected > 0 && self.tick % 30 == 0 && self.dna_bubbles.len() < 5 {
            self.dna_bubbles.push(DnaBubble {
                x: pseudo_rand(self.tick, 42, 0),
                y: pseudo_rand(self.tick, 0, 42),
                value: 1 + (severity / 3.0) as u32,
                tick_spawned: self.tick, collected: false,
            });
        }
        self.dna_bubbles.retain(|b| !b.collected && self.tick - b.tick_spawned < 200);

        // Per-region simulation
        self.simulate_regions(infectivity, severity, lethality);

        // Transport simulation
        self.simulate_transports(infectivity);

        // Spawn new transports
        self.spawn_transports();

        // Cross-border land spread
        self.cross_border_spread();

        // Border closures (human AI)
        self.update_borders(severity);

        // Multi-stage cure
        self.update_cure_system(severity);

        // Panic & lockdowns
        self.update_panic_and_lockdowns(severity);

        // Supply chain
        self.update_supply_chains();

        // Healthcare collapse check
        self.update_healthcare();

        // Synergy check
        self.check_synergies();

        // Random events
        self.random_events();

        // Record history
        if self.tick % 10 == 0 {
            for r in &mut self.regions {
                r.record_history(self.tick);
            }
        }

        // Recount
        self.recount();

        // Endgame
        self.check_endgame();
    }

    // ─────────────────────────────────────────────────────────
    // Per-region infection/death simulation
    // ─────────────────────────────────────────────────────────

    fn simulate_regions(&mut self, infectivity: f32, _severity: f32, lethality: f32) {
        let season = self.season;
        let _symp_cloaked: Vec<u16> = self.symp_cloaked_regions.iter().map(|(id, _)| *id).collect();

        for i in 0..self.regions.len() {
            let r = &self.regions[i];
            if r.infected == 0 || r.fallen { continue; }

            let base_rate = 0.0008 * infectivity as f64;

            // Climate modifier
            let climate_mod = match (r.climate, season) {
                (Climate::Tropical, _) => 1.2,
                (Climate::Temperate, Season::Winter) => 1.3, // respiratory diseases spread more
                (Climate::Temperate, Season::Summer) => 0.8,
                (Climate::Arctic, Season::Winter) => 1.1,
                (Climate::Arctic, Season::Summer) => 0.6,
                (Climate::Arid, _) => 0.9,
                _ => 1.0,
            };

            // Density modifier
            let density_mod = match r.density {
                Density::Megacity => 1.5,
                Density::Urban => 1.0,
                Density::Rural => 0.6,
            };

            // Drug resistance
            let drug_mod = if r.is_wealthy {
                let base = 0.5;
                let d1 = if self.disease.has_upgrade("drug_resistance1") { 1.5 } else { 1.0 };
                let d2 = if self.disease.has_upgrade("drug_resistance2") { 2.0 } else { 1.0 };
                base * d1 * d2
            } else {
                1.0
            };

            // Lockdown reduces spread
            let lockdown_mod = 1.0 - (r.lockdown_level * 0.5);

            // Misinformation delays lockdowns (increases spread early)
            let misinfo_mod = 1.0 + (r.misinformation * 0.3);

            // Synergy bonuses
            let mut synergy_bonus = 0.0f32;
            for syn in &self.synergies {
                if syn.unlocked {
                    // Aerosolized Dispersal: +20% in tropical
                    if syn.id == "aerosolized_dispersal" && r.climate == Climate::Tropical {
                        synergy_bonus += 0.2;
                    }
                }
            }

            let new_infected = (r.infected as f64
                * base_rate
                * climate_mod as f64
                * density_mod as f64
                * drug_mod as f64
                * lockdown_mod as f64
                * misinfo_mod as f64
                * (1.0 + synergy_bonus) as f64
                * (r.healthy() as f64 / r.population as f64)) as u64;
            let new_infected = new_infected.max(1).min(r.healthy());

            // Death rate
            let mortality_mult = r.mortality_multiplier();
            let death_rate = 0.00002 * lethality as f64 * mortality_mult as f64;
            let new_deaths = (r.infected as f64 * death_rate) as u64;
            let new_deaths = new_deaths.min(r.infected.saturating_sub(r.dead));

            // Apply
            self.regions[i].infected = (self.regions[i].infected + new_infected).min(self.regions[i].population);
            self.regions[i].dead = (self.regions[i].dead + new_deaths).min(self.regions[i].infected);

            if self.regions[i].dead >= self.regions[i].population {
                self.regions[i].fallen = true;
                self.news.push(format!("{} has fallen.", self.regions[i].name));
            }
        }

        // Clean up expired symptom cloaks
        self.symp_cloaked_regions.retain(|(_, exp)| self.tick < *exp);
    }

    // ─────────────────────────────────────────────────────────
    // Transport system
    // ─────────────────────────────────────────────────────────

    fn spawn_transports(&mut self) {
        if self.tick % 20 != 0 { return; } // Every 20 ticks

        let infected_regions: Vec<u16> = self.regions.iter()
            .filter(|r| r.infected > 0 && (r.has_airport || r.has_seaport) && !r.fallen)
            .map(|r| r.id)
            .collect();

        for &origin_id in &infected_regions {
            let origin = match self.regions.iter().find(|r| r.id == origin_id) {
                Some(r) => r,
                None => continue,
            };

            // Air flights
            if origin.has_airport && origin.air_borders_open {
                let destinations: Vec<u16> = self.regions.iter()
                    .filter(|r| r.id != origin_id && r.has_airport && r.air_borders_open && !r.fallen)
                    .map(|r| r.id)
                    .collect();

                if !destinations.is_empty() {
                    let dest_idx = (pseudo_rand(self.tick, origin_id as usize, 0) * destinations.len() as f32) as usize;
                    let dest_id = destinations[dest_idx.min(destinations.len() - 1)];

                    let infection_pct = origin.infection_pct();
                    let passengers = 100 + (pseudo_rand(self.tick, origin_id as usize, 1) * 400.0) as u64;
                    let infected = (passengers as f32 * infection_pct) as u64;

                    self.transports.push(TransportEntity {
                        id: self.next_transport_id,
                        origin: origin_id,
                        destination: dest_id,
                        transport_type: TransportType::Flight,
                        progress: 0.0,
                        infected_passengers: infected,
                        total_passengers: passengers,
                    });
                    self.next_transport_id += 1;
                }
            }

            // Cargo ships
            if origin.has_seaport && origin.sea_borders_open {
                let destinations: Vec<u16> = self.regions.iter()
                    .filter(|r| r.id != origin_id && r.has_seaport && r.sea_borders_open && !r.fallen)
                    .map(|r| r.id)
                    .collect();

                if !destinations.is_empty() {
                    let dest_idx = (pseudo_rand(self.tick, origin_id as usize, 2) * destinations.len() as f32) as usize;
                    let dest_id = destinations[dest_idx.min(destinations.len() - 1)];

                    let infection_pct = origin.infection_pct();
                    let crew = 20 + (pseudo_rand(self.tick, origin_id as usize, 3) * 80.0) as u64;
                    let infected = (crew as f32 * infection_pct) as u64;

                    self.transports.push(TransportEntity {
                        id: self.next_transport_id,
                        origin: origin_id,
                        destination: dest_id,
                        transport_type: TransportType::CargoShip,
                        progress: 0.0,
                        infected_passengers: infected,
                        total_passengers: crew,
                    });
                    self.next_transport_id += 1;
                }
            }
        }
    }

    fn simulate_transports(&mut self, _infectivity: f32) {
        let mut arrivals: Vec<(u16, u64, TransportType, u16)> = Vec::new();

        for t in &mut self.transports {
            t.progress += t.transport_type.speed();
            if t.progress >= 1.0 {
                arrivals.push((t.destination, t.infected_passengers, t.transport_type, t.origin));
            }
        }

        self.transports.retain(|t| t.progress < 1.0);

        // Process arrivals
        for (dest_id, infected, ttype, origin_id) in arrivals {
            if infected == 0 { continue; }
            let origin_name = self.regions.iter().find(|r| r.id == origin_id)
                .map(|r| r.name.clone()).unwrap_or_default();
            let tname = match ttype {
                TransportType::Flight => "flight",
                TransportType::CargoShip => "ship",
            };
            if let Some(dest) = self.regions.iter_mut().find(|r| r.id == dest_id) {
                if dest.infected == 0 {
                    dest.infected = infected.min(dest.population);
                    self.news.push(format!("Infected {tname} from {origin_name} arrived in {}!", dest.name));
                    self.events.push(GameEvent {
                        tick: self.tick,
                        message: format!("Infected {tname}: {origin_name} → {}", dest.name),
                        event_type: match ttype {
                            TransportType::Flight => EventType::FlightInfected(origin_name, dest.name.clone()),
                            TransportType::CargoShip => EventType::ShipInfected(origin_name, dest.name.clone()),
                        },
                    });
                } else {
                    dest.infected = (dest.infected + infected).min(dest.population);
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────
    // Cross-border land spread
    // ─────────────────────────────────────────────────────────

    fn cross_border_spread(&mut self) {
        let neighbor_pairs = self.get_neighbor_pairs();
        let infected_ids: Vec<u16> = self.regions.iter()
            .filter(|r| r.infected > 0 && !r.fallen)
            .map(|r| r.id)
            .collect();

        for &from_id in &infected_ids {
            let from = match self.regions.iter().find(|r| r.id == from_id) {
                Some(r) => r, None => continue,
            };
            let from_pct = from.infection_pct();
            if from_pct < 0.01 { continue; }

            for &(a, b) in &neighbor_pairs {
                let to_id = if a == from_id { b } else if b == from_id { a } else { continue };
                let _to = match self.regions.iter().find(|r| r.id == to_id) {
                    Some(r) if r.infected == 0 && r.borders_open => r,
                    _ => continue,
                };

                let chance = from_pct as f64 * 0.001;
                if pseudo_rand(self.tick, from_id as usize, to_id as usize) < chance as f32 {
                    if let Some(r) = self.regions.iter_mut().find(|r| r.id == to_id) {
                        r.infected = 1;
                        self.news.push(format!("Infection reached {}!", r.name));
                    }
                }
            }
        }
    }

    fn get_neighbor_pairs(&self) -> Vec<(u16, u16)> {
        vec![
            // North America
            (1, 2), (1, 3), (3, 4),
            // Central America → Caribbean & South America
            (4, 5), (4, 8), (5, 8), (5, 10),
            // South America
            (6, 7), (6, 8), (6, 12), (6, 13), (6, 14),
            (8, 9), (8, 15), (9, 15), (9, 12),
            (7, 13), (7, 14), (7, 11),
            (10, 16), (6, 16),
            // British Isles
            (17, 18),
            // Western Europe
            (17, 19), (19, 20), (19, 21), (19, 24), (19, 25),
            (20, 24), (20, 25), (20, 27), (20, 28), (20, 29),
            (21, 22), (23, 24), (23, 25), (23, 37),
            // Scandinavia
            (17, 26), (20, 26), (45, 26),
            // Central Europe
            (27, 28), (27, 29), (27, 31), (27, 37),
            (28, 29), (28, 30), (28, 41), (28, 42),
            (29, 30), (29, 31),
            (30, 31), (30, 41),
            (31, 32), (31, 33), (31, 34),
            // Balkans
            (32, 33), (32, 36), (32, 37),
            (33, 34), (33, 36), (33, 38), (33, 39), (33, 40),
            (34, 35), (34, 41),
            (35, 46),
            (36, 38), (36, 40),
            (38, 39), (39, 40),
            // Eastern Europe
            (41, 42), (41, 43), (41, 45),
            (42, 44), (42, 45),
            (43, 34),
            // Russia
            (45, 26), (45, 44), (45, 46), (45, 86), (45, 105), (45, 91),
            // Caucasus
            (46, 47), (46, 48), (46, 49),
            // Middle East
            (47, 48), (47, 49), (47, 50), (47, 51),
            (48, 49), (48, 90), (48, 93),
            (49, 50), (49, 51), (49, 54),
            (50, 51), (50, 54),
            (51, 52), (51, 53), (51, 54),
            (52, 53), (52, 54),
            (53, 54),
            // Middle East → Africa
            (54, 55), (55, 56),
            // North Africa
            (55, 56), (55, 75),
            (56, 57),
            (57, 58), (57, 59),
            (58, 59),
            // West Africa
            (58, 60), (58, 62),
            (60, 61), (60, 66),
            (61, 65), (61, 66),
            (62, 63), (62, 64), (62, 65),
            (63, 64), (63, 65), (63, 67), (63, 75),
            (64, 65),
            (65, 66),
            // Horn of Africa
            (55, 67), (55, 68), (55, 69),
            (67, 68), (67, 69), (67, 70), (67, 72), (67, 75),
            (68, 69), (68, 70),
            (69, 70),
            // East Africa
            (70, 71), (70, 72),
            (71, 72), (71, 74), (71, 76), (71, 81), (71, 85),
            (72, 73), (72, 76),
            (73, 76),
            (74, 76), (74, 81), (74, 83),
            // Sudan
            (75, 55), (75, 67), (75, 76), (75, 78),
            // Central Africa
            (76, 77), (76, 78), (76, 80),
            (77, 78), (77, 79), (77, 80),
            (78, 80),
            // Southern Africa
            (80, 81), (80, 82), (80, 83),
            (81, 82), (81, 83), (81, 84),
            (82, 83),
            // Central Asia
            (45, 86), (86, 87), (86, 91),
            (87, 88), (87, 89), (87, 90),
            (88, 90), (88, 48),
            (89, 90), (89, 92),
            (90, 92), (90, 93),
            // South Asia
            (92, 93), (92, 94), (92, 95), (92, 96),
            (93, 94), (93, 90),
            // Southeast Asia
            (97, 98), (97, 102), (97, 103), (97, 104), (97, 112), (97, 113),
            (98, 99), (98, 101), (98, 103),
            (99, 101), (99, 103),
            (100, 102),
            (101, 102), (101, 105),
            (102, 104),
            (103, 99), (103, 101),
            // East Asia
            (105, 45), (105, 91), (105, 101), (105, 107), (105, 108), (105, 109),
            (107, 108),
            // Oceania
            (97, 110), (97, 112), (97, 113),
            (110, 111), (110, 112), (110, 113),
        ]
    }

    // ─────────────────────────────────────────────────────────
    // Border closures (Human AI)
    // ─────────────────────────────────────────────────────────

    fn update_borders(&mut self, severity: f32) {
        if severity < 0.5 { return; }

        let neighbor_pairs = self.get_neighbor_pairs();
        let infected_ids: Vec<u16> = self.regions.iter()
            .filter(|r| r.infected > 0).map(|r| r.id).collect();

        for &(a, b) in &neighbor_pairs {
            let (infected_id, healthy_id) = if infected_ids.contains(&a) && !infected_ids.contains(&b) {
                (a, b)
            } else if infected_ids.contains(&b) && !infected_ids.contains(&a) {
                (b, a)
            } else { continue };

            let close_chance = (severity as f64 / 50.0).min(0.1)
                * self.difficulty.border_close_mult() as f64;

            // Government type affects response speed
            let gov_mult = match self.regions.iter().find(|r| r.id == healthy_id) {
                Some(r) => match r.government_type {
                    GovernmentType::Authoritarian => 1.5, // Faster response
                    GovernmentType::Democratic => 0.7,    // Slower due to debate
                    GovernmentType::Failed => 0.1,        // Barely responds
                },
                None => 1.0,
            };

            if pseudo_rand(self.tick, infected_id as usize, healthy_id as usize) < (close_chance * gov_mult) as f32 {
                if let Some(r) = self.regions.iter_mut().find(|r| r.id == healthy_id && r.borders_open) {
                    r.borders_open = false;
                    // Air borders close first
                    r.air_borders_open = false;
                    self.news.push(format!("{} closes borders!", r.name));
                    self.events.push(GameEvent {
                        tick: self.tick,
                        message: format!("{} closes borders", r.name),
                        event_type: EventType::BorderClosed(r.name.clone()),
                    });
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────
    // Multi-stage cure system
    // ─────────────────────────────────────────────────────────

    fn update_cure_system(&mut self, severity: f32) {
        let infected_count = self.regions.iter().filter(|r| r.infected > 0).count();

        // Phase transitions
        match self.cure_phase {
            CurePhase::Inactive if infected_count >= 3 && severity >= 0.5 => {
                self.cure_phase = CurePhase::Research;
                self.news.push("Cure research has begun!".into());
                self.events.push(GameEvent {
                    tick: self.tick, message: "Cure research started".into(),
                    event_type: EventType::CurePhaseChange(CurePhase::Research),
                });
            }
            CurePhase::Research if self.cure_research_progress >= 100.0 => {
                self.cure_phase = CurePhase::Trials;
                self.cure_research_progress = 100.0;
                self.news.push("Cure enters human trials!".into());
                self.events.push(GameEvent {
                    tick: self.tick, message: "Cure trials started".into(),
                    event_type: EventType::CurePhaseChange(CurePhase::Trials),
                });
            }
            CurePhase::Trials if self.cure_trials_progress >= 100.0 => {
                self.cure_phase = CurePhase::Manufacturing;
                self.cure_trials_progress = 100.0;
                self.news.push("Cure approved! Manufacturing begins.".into());
                self.events.push(GameEvent {
                    tick: self.tick, message: "Manufacturing started".into(),
                    event_type: EventType::CurePhaseChange(CurePhase::Manufacturing),
                });
            }
            CurePhase::Manufacturing if self.cure_manufacturing_progress >= 100.0 => {
                self.cure_phase = CurePhase::Distribution;
                self.cure_manufacturing_progress = 100.0;
                self.news.push("Vaccines ready! Distribution underway.".into());
                self.events.push(GameEvent {
                    tick: self.tick, message: "Distribution started".into(),
                    event_type: EventType::CurePhaseChange(CurePhase::Distribution),
                });
            }
            CurePhase::Distribution if self.cure_distribution_progress >= 100.0 => {
                self.cure_phase = CurePhase::Complete;
                self.cure_distribution_progress = 100.0;
            }
            _ => {}
        }

        // Progress within current phase
        let cure_speed = self.difficulty.cure_speed_mult();
        let cure_slowdown = 1.0 - (self.disease.cure_slowdown / 100.0);

        match self.cure_phase {
            CurePhase::Research => {
                // Research speed: wealthy countries contribute more
                let research_speed: f32 = self.regions.iter()
                    .filter(|r| r.infected > 0)
                    .map(|r| {
                        let base = if r.is_wealthy { 0.05 } else { 0.02 };
                        let dead_penalty = (1.0 - r.death_pct() * 0.5).max(0.3);
                        base * dead_penalty
                    })
                    .sum::<f32>() * cure_speed * cure_slowdown;
                self.cure_research_progress = (self.cure_research_progress + research_speed).min(100.0);
            }
            CurePhase::Trials => {
                // Trials need healthy populations in wealthy nations
                let trial_speed: f32 = self.regions.iter()
                    .filter(|r| r.is_wealthy && r.healthy() > 1000)
                    .map(|r| 0.03 * (r.healthy() as f32 / r.population as f32))
                    .sum::<f32>() * cure_speed;
                self.cure_trials_progress = (self.cure_trials_progress + trial_speed).min(100.0);
            }
            CurePhase::Manufacturing => {
                // Manufacturing depends on global manufacturing capacity
                let mfg_speed = self.global_manufacturing * 0.02 * cure_speed;
                self.cure_manufacturing_progress = (self.cure_manufacturing_progress + mfg_speed).min(100.0);
            }
            CurePhase::Distribution => {
                // Distribution depends on transport infrastructure
                let active_transports = self.regions.iter()
                    .filter(|r| r.has_airport && !r.fallen)
                    .count() as f32;
                let dist_speed = (active_transports / 50.0) * 0.02 * cure_speed;
                self.cure_distribution_progress = (self.cure_distribution_progress + dist_speed).min(100.0);

                // Vaccinate people in regions with airports
                for r in &mut self.regions {
                    if r.has_airport && !r.fallen && r.infected > 0 {
                        let doses_per_tick = (r.population as f64 * 0.0001) as u64;
                        r.vaccinated = (r.vaccinated + doses_per_tick).min(r.population);
                    }
                }
            }
            _ => {}
        }

        // Overall cure progress (weighted)
        self.cure_overall = match self.cure_phase {
            CurePhase::Inactive => 0.0,
            CurePhase::Research => self.cure_research_progress * 0.25,
            CurePhase::Trials => 25.0 + self.cure_trials_progress * 0.25,
            CurePhase::Manufacturing => 50.0 + self.cure_manufacturing_progress * 0.25,
            CurePhase::Distribution => 75.0 + self.cure_distribution_progress * 0.25,
            CurePhase::Complete => 100.0,
        };
    }

    // ─────────────────────────────────────────────────────────
    // Panic & Lockdowns (Human AI)
    // ─────────────────────────────────────────────────────────

    fn update_panic_and_lockdowns(&mut self, severity: f32) {
        for r in &mut self.regions {
            if r.fallen { continue; }

            // Panic rises with local infection and severity
            let local_infection = r.infection_pct();
            let panic_rise = local_infection * severity * 0.001;
            let panic_fall = 0.0005; // Panic slowly decreases
            r.panic = (r.panic + panic_rise - panic_fall).clamp(0.0, 1.0);

            // Misinformation reduces panic (denial) but delays response
            if r.misinformation > 0.3 {
                r.panic *= 0.8;
            }

            // Government response based on panic + government type
            let lockdown_threshold = match r.government_type {
                GovernmentType::Authoritarian => 0.2,
                GovernmentType::Democratic => 0.4,
                GovernmentType::Failed => 0.9,
            };

            if r.panic > lockdown_threshold {
                let lockdown_speed = match r.government_type {
                    GovernmentType::Authoritarian => 0.02,
                    GovernmentType::Democratic => 0.005,
                    GovernmentType::Failed => 0.001,
                };
                r.lockdown_level = (r.lockdown_level + lockdown_speed).min(1.0);
            } else {
                r.lockdown_level = (r.lockdown_level - 0.001).max(0.0);
            }

            // Misinformation wave (random)
            if self.tick % 500 == 0 && pseudo_rand(self.tick, r.id as usize, 77) < 0.1 {
                r.misinformation = (r.misinformation + 0.2).min(1.0);
                self.news.push(format!("Misinformation spreads in {}!", r.name));
                self.events.push(GameEvent {
                    tick: self.tick,
                    message: format!("Misinformation in {}", r.name),
                    event_type: EventType::MisinformationWave(r.name.clone()),
                });
            }
        }

        self.global_panic = self.regions.iter().map(|r| r.panic).sum::<f32>()
            / self.regions.len() as f32;
    }

    // ─────────────────────────────────────────────────────────
    // Supply Chain
    // ─────────────────────────────────────────────────────────

    fn update_supply_chains(&mut self) {
        // Global manufacturing = weighted average of manufacturing capacity * health
        let total_mfg: f32 = self.regions.iter()
            .map(|r| r.manufacturing_capacity * (r.healthy() as f32 / r.population as f32))
            .sum();
        self.global_manufacturing = total_mfg / self.regions.len() as f32;

        let total_agr: f32 = self.regions.iter()
            .map(|r| r.agricultural_capacity * (r.healthy() as f32 / r.population as f32))
            .sum();
        self.global_agriculture = total_agr / self.regions.len() as f32;

        // Supply chain collapse events
        if self.global_manufacturing < 0.3 && self.tick % 200 == 0 {
            self.news.push("Global supply chain collapse! Medical shortages.".into());
            self.events.push(GameEvent {
                tick: self.tick, message: "Supply chain collapse".into(),
                event_type: EventType::SupplyChainBreak("Global".into()),
            });
        }
    }

    // ─────────────────────────────────────────────────────────
    // Healthcare
    // ─────────────────────────────────────────────────────────

    fn update_healthcare(&mut self) {
        for i in 0..self.regions.len() {
            let overwhelmed = self.regions[i].is_overwhelmed();
            if overwhelmed && !self.regions[i].healthcare_collapse {
                self.regions[i].healthcare_collapse = true;
                let name = self.regions[i].name.clone();
                self.news.push(format!("Healthcare system collapses in {name}!"));
                self.events.push(GameEvent {
                    tick: self.tick, message: format!("Healthcare collapse: {name}"),
                    event_type: EventType::HealthcareCollapse(name),
                });
            }
        }
    }

    // ─────────────────────────────────────────────────────────
    // Synergies
    // ─────────────────────────────────────────────────────────

    fn check_synergies(&mut self) {
        for syn in &mut self.synergies {
            if syn.unlocked { continue; }
            if syn.requires.iter().all(|req| self.disease.has_upgrade(req)) {
                syn.unlocked = true;
                self.disease.total_infectivity += syn.bonus_infectivity;
                self.disease.total_severity += syn.bonus_severity;
                self.disease.total_lethality += syn.bonus_lethality;
                self.news.push(format!("Synergy unlocked: {}!", syn.name));

                // Mass Hysteria: collapse healthcare in all infected regions
                if syn.id == "mass_hysteria" {
                    for r in &mut self.regions {
                        if r.infected > 0 {
                            r.healthcare_collapse = true;
                        }
                    }
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────
    // Random Events
    // ─────────────────────────────────────────────────────────

    fn random_events(&mut self) {
        if self.tick % 100 != 0 { return; }

        let roll = pseudo_rand(self.tick, 99, 99);
        if roll < 0.15 {
            // Sports event
            let infected: Vec<u16> = self.regions.iter()
                .filter(|r| r.infected > 0).map(|r| r.id).collect();
            if let Some(&id) = infected.first() {
                if let Some(r) = self.regions.iter_mut().find(|r| r.id == id) {
                    let boost = (r.population as f64 * 0.001) as u64;
                    r.infected = (r.infected + boost).min(r.population);
                    self.news.push(format!("Sports event in {} — infection spikes!", r.name));
                }
            }
        } else if roll < 0.25 {
            self.cure_research_progress += 2.0;
            self.news.push("Research funding boost!".into());
        }
    }

    // ─────────────────────────────────────────────────────────
    // Tactical Abilities
    // ─────────────────────────────────────────────────────────

    pub fn use_tactical(&mut self, ability_id: &str, target_region: u16) -> bool {
        if let Some(ability) = self.tactical_abilities.iter_mut().find(|a| a.id == ability_id) {
            if self.dna_points < ability.cost || self.tick - ability.last_used < ability.cooldown_ticks {
                return false;
            }
            self.dna_points -= ability.cost;
            ability.last_used = self.tick;

            match ability_id {
                "spore_strike" => {
                    if let Some(r) = self.regions.iter_mut().find(|r| r.id == target_region) {
                        r.infected = (r.infected + 100).min(r.population);
                        self.news.push(format!("Spore strike on {}!", r.name));
                    }
                }
                "symptom_cloak" => {
                    self.symp_cloaked_regions.push((target_region, self.tick + 100));
                    self.news.push("Symptoms cloaked in a region.".into());
                }
                "infectious_surge" => {
                    if let Some(r) = self.regions.iter_mut().find(|r| r.id == target_region) {
                        let boost = (r.population as f64 * 0.005) as u64;
                        r.infected = (r.infected + boost).min(r.population);
                        self.news.push(format!("Mass gathering in {} — infections surge!", r.name));
                    }
                }
                _ => {}
            }
            return true;
        }
        false
    }

    // ─────────────────────────────────────────────────────────
    // Endgame
    // ─────────────────────────────────────────────────────────

    fn check_endgame(&mut self) {
        let has_healthy = self.regions.iter().any(|r| r.healthy() > 0);
        if !has_healthy && self.phase == GamePhase::Playing {
            self.phase = GamePhase::Won;
            self.news.push("Humanity has fallen. You win.".into());
        }

        // Lose only when cure is fully distributed (not just researched)
        if self.cure_phase == CurePhase::Complete && self.phase == GamePhase::Playing {
            // Check if infrastructure is destroyed — if so, can still win
            if self.global_manufacturing > 0.1 {
                self.phase = GamePhase::Lost;
                self.news.push("The cure has been distributed. You lose.".into());
            } else {
                self.news.push("Cure completed but infrastructure collapsed! Keep fighting.".into());
            }
        }
    }

    fn recount(&mut self) {
        self.total_infected = 0;
        self.total_dead = 0;
        self.total_healthy = 0;
        for r in &self.regions {
            self.total_infected += r.infected;
            self.total_dead += r.dead;
            self.total_healthy += r.healthy();
        }
    }

    pub fn collect_bubble(&mut self, x: f32, y: f32) -> bool {
        for bubble in &mut self.dna_bubbles {
            if bubble.collected { continue; }
            if (bubble.x - x).abs() < 0.03 && (bubble.y - y).abs() < 0.03 {
                bubble.collected = true;
                self.dna_points += bubble.value;
                return true;
            }
        }
        false
    }
}

fn pseudo_rand(tick: u64, a: usize, b: usize) -> f32 {
    let mut h = tick.wrapping_mul(374761393)
        .wrapping_add((a as u64).wrapping_mul(668265263))
        .wrapping_add((b as u64).wrapping_mul(2147483647));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h = h ^ (h >> 16);
    (h & 0xFFFF) as f32 / 65535.0
}
