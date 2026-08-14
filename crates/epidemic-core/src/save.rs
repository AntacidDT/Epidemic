use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::disease::PathogenType;
use crate::world::{Difficulty, GamePhase, Season};

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub tick: u64,
    pub game_speed: u32,
    pub dna_points: u32,
    pub phase: String,
    pub difficulty: String,
    pub pathogen_type: String,
    pub disease_name: String,
    pub cure_overall: f32,
    pub cure_phase: String,
    pub global_panic: f32,
    pub total_infected: u64,
    pub total_dead: u64,
    pub total_healthy: u64,
    pub regions: Vec<RegionSave>,
    pub upgrades_unlocked: Vec<String>,
    pub synergies_unlocked: Vec<String>,
    pub news: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RegionSave {
    pub id: u16,
    pub infected: u64,
    pub dead: u64,
    pub borders_open: bool,
    pub air_borders_open: bool,
    pub sea_borders_open: bool,
    pub panic: f32,
    pub misinformation: f32,
    pub lockdown_level: f32,
    pub healthcare_collapse: bool,
    pub fallen: bool,
    pub cure_progress: f32,
    pub vaccinated: u64,
}

impl SaveData {
    pub fn from_world(world: &crate::world::World) -> Self {
        Self {
            version: 1,
            tick: world.tick,
            game_speed: world.game_speed,
            dna_points: world.dna_points,
            phase: format!("{:?}", world.phase),
            difficulty: format!("{:?}", world.difficulty),
            pathogen_type: format!("{:?}", world.disease.pathogen_type),
            disease_name: world.disease_name.clone(),
            cure_overall: world.cure_overall,
            cure_phase: format!("{:?}", world.cure_phase),
            global_panic: world.global_panic,
            total_infected: world.total_infected,
            total_dead: world.total_dead,
            total_healthy: world.total_healthy,
            regions: world.regions.iter().map(|r| RegionSave {
                id: r.id, infected: r.infected, dead: r.dead,
                borders_open: r.borders_open, air_borders_open: r.air_borders_open,
                sea_borders_open: r.sea_borders_open, panic: r.panic,
                misinformation: r.misinformation, lockdown_level: r.lockdown_level,
                healthcare_collapse: r.healthcare_collapse, fallen: r.fallen,
                cure_progress: r.cure_progress, vaccinated: r.vaccinated,
            }).collect(),
            upgrades_unlocked: world.disease.upgrades.keys().cloned().collect(),
            synergies_unlocked: world.synergies.iter().filter(|s| s.unlocked).map(|s| s.id.to_string()).collect(),
            news: world.news.clone(),
        }
    }

    pub fn apply_to_world(&self, world: &mut crate::world::World) {
        world.tick = self.tick;
        world.game_speed = self.game_speed;
        world.dna_points = self.dna_points;
        world.cure_overall = self.cure_overall;
        world.global_panic = self.global_panic;
        world.total_infected = self.total_infected;
        world.total_dead = self.total_dead;
        world.total_healthy = self.total_healthy;
        world.news = self.news.clone();

        // Restore phase
        world.phase = match self.phase.as_str() {
            "TitleScreen" => GamePhase::TitleScreen,
            "PathogenSelect" => GamePhase::PathogenSelect,
            "DifficultySelect" => GamePhase::DifficultySelect,
            "SelectOrigin" => GamePhase::SelectOrigin,
            "Playing" => GamePhase::Playing,
            "Won" => GamePhase::Won,
            "Lost" => GamePhase::Lost,
            _ => GamePhase::Playing,
        };

        // Restore difficulty
        world.difficulty = match self.difficulty.as_str() {
            "Casual" => Difficulty::Casual,
            "Normal" => Difficulty::Normal,
            "Brutal" => Difficulty::Brutal,
            "MegaBrutal" => Difficulty::MegaBrutal,
            _ => Difficulty::Normal,
        };

        // Restore disease
        let ptype = match self.pathogen_type.as_str() {
            "Bacteria" => PathogenType::Bacteria,
            "Virus" => PathogenType::Virus,
            "Fungus" => PathogenType::Fungus,
            "Parasite" => PathogenType::Parasite,
            "Prion" => PathogenType::Prion,
            "NanoVirus" => PathogenType::NanoVirus,
            "BioWeapon" => PathogenType::BioWeapon,
            _ => PathogenType::Bacteria,
        };
        world.init_disease(&self.disease_name, ptype);

        // Restore upgrades
        for upgrade_id in &self.upgrades_unlocked {
            if let Some(upgrade) = world.upgrades.iter().find(|u| u.id == upgrade_id.as_str()).cloned() {
                world.disease.unlock(&upgrade);
            }
        }

        // Restore synergies
        for syn_id in &self.synergies_unlocked {
            if let Some(syn) = world.synergies.iter_mut().find(|s| s.id == syn_id.as_str()) {
                syn.unlocked = true;
            }
        }

        // Restore regions
        for rs in &self.regions {
            if let Some(r) = world.regions.iter_mut().find(|r| r.id == rs.id) {
                r.infected = rs.infected;
                r.dead = rs.dead;
                r.borders_open = rs.borders_open;
                r.air_borders_open = rs.air_borders_open;
                r.sea_borders_open = rs.sea_borders_open;
                r.panic = rs.panic;
                r.misinformation = rs.misinformation;
                r.lockdown_level = rs.lockdown_level;
                r.healthcare_collapse = rs.healthcare_collapse;
                r.fallen = rs.fallen;
                r.cure_progress = rs.cure_progress;
                r.vaccinated = rs.vaccinated;
            }
        }
    }
}

pub fn save_game(world: &crate::world::World, path: &Path) -> Result<(), String> {
    let data = SaveData::from_world(world);
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_game(path: &Path) -> Result<SaveData, String> {
    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let data: SaveData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(data)
}

/// Calculate game score based on performance.
pub fn calculate_score(world: &crate::world::World) -> GameScore {
    let total_pop = world.regions.iter().map(|r| r.population).sum::<u64>();

    // Time bonus: faster = more points
    let time_bonus = if world.tick > 0 { (100000.0 / world.tick as f32) as u32 } else { 0 };

    // Disease score: % killed
    let kill_pct = world.total_dead as f32 / total_pop as f32;
    let disease_score = (kill_pct * 1000.0) as u32;

    // Cure penalty: lower cure = more points
    let cure_penalty = (world.cure_overall * 5.0) as u32;

    // Difficulty multiplier
    let diff_mult = match world.difficulty {
        Difficulty::Casual => 0.5,
        Difficulty::Normal => 1.0,
        Difficulty::Brutal => 2.0,
        Difficulty::MegaBrutal => 3.0,
    };

    let raw_score = (time_bonus + disease_score).saturating_sub(cure_penalty);
    let final_score = (raw_score as f32 * diff_mult) as u32;

    // Biohazard rating (1-5)
    let biohazards = if final_score >= 5000 { 5 }
        else if final_score >= 3000 { 4 }
        else if final_score >= 1500 { 3 }
        else if final_score >= 500 { 2 }
        else { 1 };

    GameScore { time_bonus, disease_score, cure_penalty, diff_mult, raw_score, final_score, biohazards }
}

pub struct GameScore {
    pub time_bonus: u32,
    pub disease_score: u32,
    pub cure_penalty: u32,
    pub diff_mult: f32,
    pub raw_score: u32,
    pub final_score: u32,
    pub biohazards: u8,
}
