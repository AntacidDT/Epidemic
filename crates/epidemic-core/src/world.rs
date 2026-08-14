use crate::disease::{Disease, PathogenType, Upgrade, all_upgrades};
use crate::regions::{self, Region};
use crate::svg_parser;

pub struct World {
    pub regions: Vec<Region>,
    pub svg_lookup: Vec<u16>,     // pixel -> region ID (0 = ocean)
    pub lookup_w: usize,
    pub lookup_h: usize,
    pub tick: u64,
    pub game_speed: u32,          // 1x, 2x, 3x
    pub dna_points: u32,
    pub total_infected: u64,
    pub total_dead: u64,
    pub total_healthy: u64,
    pub cure_progress: f32,       // global average
    pub news: Vec<String>,
    pub phase: GamePhase,
    pub selected_region: Option<u16>,
    pub disease: Disease,
    pub upgrades: Vec<Upgrade>,   // all available upgrades
    pub events: Vec<GameEvent>,
    pub dna_bubbles: Vec<DnaBubble>,
    pub difficulty: Difficulty,
    pub disease_name: String,
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
        match self {
            Self::Casual => 0.5,
            Self::Normal => 1.0,
            Self::Brutal => 1.5,
            Self::MegaBrutal => 2.0,
        }
    }

    pub fn border_close_mult(&self) -> f32 {
        match self {
            Self::Casual => 0.5,
            Self::Normal => 1.0,
            Self::Brutal => 1.5,
            Self::MegaBrutal => 2.0,
        }
    }
}

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
    CureMilestone(f32),
    SportsEvent(String),
    InfectedPlane(String, String),
    InfectedShip(String, String),
    ResearchBoost,
}

#[derive(Debug, Clone)]
pub struct DnaBubble {
    pub x: f32,       // 0.0-1.0 normalized
    pub y: f32,       // 0.0-1.0 normalized
    pub value: u32,
    pub tick_spawned: u64,
    pub collected: bool,
}

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

impl World {
    /// Create a new world from the SVG file.
    pub fn new(svg_content: &str) -> Self {
        let regions = regions::build_regions();

        // Build SVG code -> region ID map
        let code_to_region = regions::svg_code_to_region_map(&regions);

        // Rasterize SVG to lookup table
        let lookup_w = 800;
        let lookup_h = 400;
        let (raw_lookup, svg_countries) = svg_parser::rasterize_svg_to_lookup(svg_content, lookup_w, lookup_h);

        let filled = raw_lookup.iter().filter(|&&v| v > 0).count();
        println!("SVG countries: {}, filled pixels: {}/{}", svg_countries.len(), filled, raw_lookup.len());

        // Map raw SVG country IDs to region IDs
        let mut svg_code_list: Vec<String> = svg_countries.iter().map(|c| c.code.clone()).collect();
        svg_code_list.insert(0, "".to_string()); // index 0 = ocean

        let mut lookup = vec![0u16; lookup_w * lookup_h];
        for (i, &raw_id) in raw_lookup.iter().enumerate() {
            if raw_id == 0 {
                lookup[i] = 0;
            } else if let Some(code) = svg_code_list.get(raw_id as usize) {
                lookup[i] = code_to_region.get(code).copied().unwrap_or(0);
            }
        }

        let total_pop: u64 = regions.iter().map(|r| r.population).sum();

        Self {
            regions,
            svg_lookup: lookup,
            lookup_w,
            lookup_h,
            tick: 0,
            game_speed: 1,
            dna_points: 0,
            total_infected: 0,
            total_dead: 0,
            total_healthy: total_pop,
            cure_progress: 0.0,
            news: Vec::new(),
            phase: GamePhase::SelectOrigin,
            selected_region: None,
            disease: Disease::new("Unknown", PathogenType::Bacteria),
            upgrades: all_upgrades(),
            events: Vec::new(),
            dna_bubbles: Vec::new(),
            difficulty: Difficulty::Normal,
            disease_name: "Epidemic".to_string(),
        }
    }

    pub fn init_disease(&mut self, name: &str, pathogen_type: PathogenType) {
        self.disease = Disease::new(name, pathogen_type);
        self.disease_name = name.to_string();
    }

    /// Start outbreak in a region by ID.
    pub fn start_outbreak(&mut self, region_id: u16) {
        if let Some(region) = self.regions.iter_mut().find(|r| r.id == region_id) {
            region.infected = 1;
            let name = region.name.clone();
            self.news.push(format!("Outbreak detected in {name}!"));
            self.phase = GamePhase::Playing;
            self.selected_region = Some(region_id);
        }
    }

    /// Get region at pixel coordinates in the lookup table.
    pub fn region_at_pixel(&self, px: usize, py: usize) -> Option<&Region> {
        if px >= self.lookup_w || py >= self.lookup_h {
            return None;
        }
        let id = self.svg_lookup[py * self.lookup_w + px];
        if id == 0 {
            return None;
        }
        self.regions.iter().find(|r| r.id == id)
    }

    /// Advance simulation by one tick.
    pub fn advance(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }

        self.tick += 1;

        let infectivity = self.disease.effective_infectivity();
        let severity = self.disease.effective_severity();
        let lethality = self.disease.effective_lethality();

        // DNA points: earned passively + bubbles
        if self.total_infected > 0 && self.tick % 50 == 0 {
            self.dna_points += 1 + (self.total_infected / 10_000_000).min(5) as u32;
        }

        // Spawn DNA bubbles
        if self.total_infected > 0 && self.tick % 30 == 0 && self.dna_bubbles.len() < 5 {
            let bubble = DnaBubble {
                x: pseudo_rand(self.tick, 42, 0) as f32,
                y: pseudo_rand(self.tick, 0, 42) as f32,
                value: 1 + (severity / 3.0) as u32,
                tick_spawned: self.tick,
                collected: false,
            };
            self.dna_bubbles.push(bubble);
        }

        // Remove old bubbles (expire after 200 ticks)
        self.dna_bubbles.retain(|b| !b.collected && self.tick - b.tick_spawned < 200);

        // Country-level infection spread
        let mut infections: Vec<(u16, u64)> = Vec::new();
        let mut deaths: Vec<(u16, u64)> = Vec::new();

        for region in &self.regions {
            if region.infected == 0 || region.fallen {
                continue;
            }

            // Infection rate scales with infectivity stat
            let base_rate = 0.0008 * infectivity as f64;
            // Drug resistance: wealthy countries have natural resistance
            let is_wealthy = matches!(region.code.as_str(), "US" | "GB" | "DE" | "FR" | "JP" | "KR" | "AU" | "CA" | "IT" | "ES" | "NL" | "SE" | "CH" | "WE" | "NE");
            let drug_resistance = if is_wealthy { 0.5 } else { 1.0 };
            let drug_bonus = if self.disease.has_upgrade("drug_resistance1") && is_wealthy { 1.5 } else { 1.0 };
            let drug_bonus2 = if self.disease.has_upgrade("drug_resistance2") && is_wealthy { 2.0 } else { 1.0 };

            let new_infected = (region.infected as f64
                * base_rate
                * drug_resistance
                * drug_bonus
                * drug_bonus2
                * (region.healthy() as f64 / region.population as f64))
                as u64;
            let new_infected = new_infected.max(1).min(region.healthy());

            if new_infected > 0 {
                infections.push((region.id, new_infected));
            }

            // Deaths scale with lethality
            let death_rate = 0.00002 * lethality as f64;
            let new_deaths = (region.infected as f64 * death_rate) as u64;
            let new_deaths = new_deaths.min(region.infected.saturating_sub(region.dead));

            if new_deaths > 0 {
                deaths.push((region.id, new_deaths));
            }
        }

        // Apply infections
        for (id, count) in infections {
            if let Some(r) = self.regions.iter_mut().find(|r| r.id == id) {
                let was_empty = r.infected == 0;
                r.infected = (r.infected + count).min(r.population);
                if was_empty && r.infected > 0 {
                    self.events.push(GameEvent {
                        tick: self.tick,
                        message: format!("Infection reached {}!", r.name),
                        event_type: EventType::NewCountry(r.name.clone()),
                    });
                    self.news.push(format!("Infection reached {}!", r.name));
                }
            }
        }

        // Apply deaths
        for (id, count) in deaths {
            if let Some(r) = self.regions.iter_mut().find(|r| r.id == id) {
                r.dead = (r.dead + count).min(r.infected);
                if r.dead >= r.population {
                    r.fallen = true;
                    self.news.push(format!("{} has fallen.", r.name));
                }
            }
        }

        // Cross-border spread
        self.cross_border_spread();

        // Border closures based on severity
        self.update_borders(severity);

        // Cure research
        self.update_cure(severity);

        // Random events
        self.random_events();

        // Recount
        self.recount();

        // Check endgame
        self.check_endgame();
    }

    fn cross_border_spread(&mut self) {
        // Simplified: each infected region has a chance to infect neighboring regions
        // We use the region list order as a proxy for adjacency (not perfect but functional)
        let infected_ids: Vec<u16> = self.regions.iter()
            .filter(|r| r.infected > 0 && !r.fallen)
            .map(|r| r.id)
            .collect();

        // Neighbor pairs (simplified — large regions are adjacent to their neighbors in the list)
        let neighbor_pairs = self.get_neighbor_pairs();

        for &from_id in &infected_ids {
            let from = match self.regions.iter().find(|r| r.id == from_id) {
                Some(r) => r,
                None => continue,
            };

            let from_pct = from.infection_pct();
            if from_pct < 0.01 {
                continue; // need at least 1% infected to spread
            }

            for &(a, b) in &neighbor_pairs {
                let to_id = if a == from_id { b } else if b == from_id { a } else { continue };

                let _to = match self.regions.iter().find(|r| r.id == to_id) {
                    Some(r) if r.infected == 0 && r.borders_open => r,
                    _ => continue,
                };

                // Chance of cross-border infection
                let chance = from_pct as f64 * 0.001;
                if pseudo_rand(self.tick, from_id as usize, to_id as usize) < chance as f32 {
                    let seed = 1u64;
                    if let Some(r) = self.regions.iter_mut().find(|r| r.id == to_id) {
                        r.infected = seed;
                        self.news.push(format!("Infection reached {}!", r.name));
                    }
                }
            }
        }
    }

    fn get_neighbor_pairs(&self) -> Vec<(u16, u16)> {
        // Adjacency based on geographic proximity in the region list
        // This is a simplification — a proper adjacency graph would be better
        let mut pairs = Vec::new();

        // North America chain
        pairs.extend_from_slice(&[(1, 2), (1, 3), (3, 4)]);
        // South America
        pairs.extend_from_slice(&[(5, 6), (5, 7), (5, 10), (7, 8), (8, 10)]);
        // Europe
        pairs.extend_from_slice(&[
            (11, 12), (12, 13), (13, 14), (13, 15), (14, 16), (12, 16),
            (13, 17), (13, 18), (13, 19), (19, 20), (20, 21), (13, 21),
        ]);
        // Russia
        pairs.extend_from_slice(&[(13, 22), (18, 22), (19, 22), (21, 22)]);
        // Middle East
        pairs.extend_from_slice(&[(22, 23), (23, 24), (24, 25), (25, 26), (26, 27)]);
        // Africa
        pairs.extend_from_slice(&[
            (27, 28), (28, 29), (29, 30), (30, 31),
            (31, 32), (32, 33), (33, 34),
            (28, 35), (35, 36), (36, 37), (37, 38),
            (34, 39), (39, 40), (40, 41), (41, 42),
        ]);
        // Asia
        pairs.extend_from_slice(&[
            (22, 43), (43, 44), (44, 45), (45, 46), (46, 47), (47, 48),
            (48, 49), (49, 50), (50, 51), (51, 52), (52, 53), (53, 54), (54, 55),
            (22, 56), (56, 57), (56, 58), (58, 59), (56, 60),
            (49, 61), (55, 49),
        ]);
        // Oceania
        pairs.extend_from_slice(&[(61, 62), (61, 63), (49, 63)]);

        pairs
    }

    fn update_cure(&mut self, severity: f32) {
        let infected_count = self.regions.iter().filter(|r| r.infected > 0).count();
        if infected_count < 3 || severity < 0.5 {
            return;
        }

        let cure_speed_modifier = 1.0 - (self.disease.cure_slowdown / 100.0);

        for region in &mut self.regions {
            if region.cure_progress >= 100.0 {
                continue;
            }
            // Wealthy countries research faster
            let is_wealthy = matches!(region.code.as_str(), "US" | "GB" | "DE" | "FR" | "JP" | "KR" | "AU" | "CA" | "WE" | "NE");
            let research_speed = if is_wealthy { 0.008 } else { 0.003 };
            // Severity drives cure urgency
            let severity_factor = (severity / 10.0).min(2.0);
            // More dead researchers = slower cure
            let dead_pct = region.dead as f32 / region.population as f32;
            let dead_penalty = (1.0 - dead_pct * 0.5).max(0.3);

            region.cure_progress += research_speed * severity_factor * dead_penalty * cure_speed_modifier;
            region.cure_progress = region.cure_progress.min(100.0);
        }

        let total: f32 = self.regions.iter().map(|r| r.cure_progress).sum();
        let new_progress = total / self.regions.len() as f32;

        // News for cure milestones
        let milestones = [10.0, 25.0, 50.0, 75.0, 90.0];
        for &m in &milestones {
            if self.cure_progress < m && new_progress >= m {
                self.news.push(format!("Cure research at {:.0}%!", m));
                self.events.push(GameEvent {
                    tick: self.tick,
                    message: format!("Cure research at {:.0}%", m),
                    event_type: EventType::CureMilestone(m),
                });
            }
        }
        self.cure_progress = new_progress;
    }

    fn update_borders(&mut self, severity: f32) {
        // Countries close borders when severity is high enough and infection is nearby
        if severity < 1.0 {
            return;
        }

        let neighbor_pairs = self.get_neighbor_pairs();
        let infected_ids: Vec<u16> = self.regions.iter()
            .filter(|r| r.infected > 0)
            .map(|r| r.id)
            .collect();

        for &(a, b) in &neighbor_pairs {
            let (infected_id, healthy_id) = if infected_ids.contains(&a) && !infected_ids.contains(&b) {
                (a, b)
            } else if infected_ids.contains(&b) && !infected_ids.contains(&a) {
                (b, a)
            } else {
                continue;
            };

            // Chance of border closure based on severity
            let close_chance = (severity as f64 / 50.0).min(0.1);
            if pseudo_rand(self.tick, infected_id as usize, healthy_id as usize) < close_chance as f32 {
                if let Some(r) = self.regions.iter_mut().find(|r| r.id == healthy_id && r.borders_open) {
                    r.borders_open = false;
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

    fn random_events(&mut self) {
        if self.tick % 100 != 0 {
            return;
        }

        let roll = pseudo_rand(self.tick, 99, 99);
        if roll < 0.15 {
            // Sports event — infection boost in a random infected country
            let infected: Vec<u16> = self.regions.iter()
                .filter(|r| r.infected > 0)
                .map(|r| r.id)
                .collect();
            if let Some(&id) = infected.first() {
                if let Some(r) = self.regions.iter_mut().find(|r| r.id == id) {
                    let boost = (r.population as f64 * 0.001) as u64;
                    r.infected = (r.infected + boost).min(r.population);
                    self.news.push(format!("Sports event in {} — infection spikes!", r.name));
                    self.events.push(GameEvent {
                        tick: self.tick,
                        message: format!("Sports event in {}", r.name),
                        event_type: EventType::SportsEvent(r.name.clone()),
                    });
                }
            }
        } else if roll < 0.25 {
            // Research funding boost
            self.cure_progress += 2.0;
            self.cure_progress = self.cure_progress.min(100.0);
            self.news.push("Research funding boost — cure accelerates!".into());
            self.events.push(GameEvent {
                tick: self.tick,
                message: "Research funding boost".into(),
                event_type: EventType::ResearchBoost,
            });
        }
    }

    pub fn collect_bubble(&mut self, x: f32, y: f32) -> bool {
        for bubble in &mut self.dna_bubbles {
            if bubble.collected {
                continue;
            }
            let dx = (bubble.x - x).abs();
            let dy = (bubble.y - y).abs();
            if dx < 0.03 && dy < 0.03 {
                bubble.collected = true;
                self.dna_points += bubble.value;
                return true;
            }
        }
        false
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

    fn check_endgame(&mut self) {
        let has_healthy = self.regions.iter().any(|r| r.healthy() > 0);
        if !has_healthy && self.phase == GamePhase::Playing {
            self.phase = GamePhase::Won;
            self.news.push("Humanity has fallen. You win.".into());
        }

        if self.cure_progress >= 100.0 && self.phase == GamePhase::Playing {
            self.phase = GamePhase::Lost;
            self.news.push("The cure has been completed. You lose.".into());
        }
    }

    /// Build the GPU instance data for rendering.
    /// Each pixel in the lookup gets: [u, v, region_id, infection_pct]
    pub fn render_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.lookup_w * self.lookup_h * 4);
        let w = self.lookup_w as f32;
        let h = self.lookup_h as f32;

        for y in 0..self.lookup_h {
            for x in 0..self.lookup_w {
                let region_id = self.svg_lookup[y * self.lookup_w + x];
                let infection_pct = if region_id > 0 {
                    self.regions
                        .iter()
                        .find(|r| r.id == region_id)
                        .map(|r| r.infection_pct())
                        .unwrap_or(0.0)
                } else {
                    0.0 // ocean
                };

                data.push(x as f32 / w);      // u
                data.push(y as f32 / h);      // v
                data.push(region_id as f32);  // region_id
                data.push(infection_pct);     // infection %
            }
        }
        data
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
