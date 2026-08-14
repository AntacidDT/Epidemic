# Epidemic NS — Full Codebase Reference

## Table of Contents
1. [Project Structure](#project-structure)
2. [Workspace Configuration](#workspace-configuration)
3. [epidemic-core Crate](#epidemic-core-crate)
4. [epidemic-render Crate](#epidemic-render-crate)
5. [epidemic-app Crate](#epidemic-app-crate)
6. [Shaders](#shaders)
7. [Assets](#assets)
8. [Android Configuration](#android-configuration)
9. [Dependencies](#dependencies)
10. [Game Flow](#game-flow)
11. [Simulation Model](#simulation-model)

---

## Project Structure

```
epidemic-ns/
├── Cargo.toml                          # Workspace root (8 lines)
├── Cargo.lock                          # Dependency lockfile
├── .gitignore                          # Ignores /target, *.swp, .DS_Store
├── x.toml                             # Android build config (16 lines)
├── CODEBASE.md                         # This file
├── crates/
│   ├── epidemic-core/                  # Pure game logic (no rendering)
│   │   ├── Cargo.toml                  # Dependencies: resvg, usvg, tiny-skia
│   │   └── src/
│   │       ├── lib.rs                  # Module exports (11 lines)
│   │       ├── disease.rs              # Disease model (379 lines)
│   │       ├── world.rs                # World simulation (579 lines)
│   │       ├── regions.rs              # Region definitions (187 lines)
│   │       ├── svg_parser.rs           # SVG rasterizer (210 lines)
│   │       ├── sim.rs                  # GameState stub (31 lines)
│   │       └── map_data.rs             # Old grid map (unused)
│   ├── epidemic-render/                # wgpu rendering + egui UI
│   │   ├── Cargo.toml                  # Dependencies: wgpu, winit, egui, image
│   │   ├── src/
│   │   │   └── lib.rs                  # Renderer + HUD (1072 lines)
│   │   └── shaders/
│   │       ├── map.wgsl                # Full-screen map shader (55 lines)
│   │       ├── grid.wgsl               # Old grid shader (unused) (85 lines)
│   │       └── triangle.wgsl           # Old triangle shader (unused) (19 lines)
│   └── epidemic-app/                   # Platform entry points
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs                  # Shared run() (5 lines)
│       │   └── main.rs                 # Linux entry (3 lines)
└── Assets/
    ├── EPIDEMIC.png                    # Game logo
    ├── world.svg                       # World map SVG (977 lines)
    ├── plane737.svg                    # Plane icon
    ├── ship.svg                        # Ship icon
    └── biohazard.svg                   # Biohazard icon
```

---

## Workspace Configuration

### Cargo.toml (root)
```toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
```

- **members**: All crates in `crates/` are workspace members
- **resolver = "3"**: Rust 2024 edition resolver
- **edition = "2024"**: Latest Rust edition

### x.toml (Android)
```toml
[app]
name = "Epidemic NS"
identifier = "com.epidemicns.app"
icon = "res/icon"
label = "Epidemic NS"
assets = ["res"]

[android]
gradle-plugins = ["org.jetbrains.kotlin.android:2.1.0"]

[android.sdk]
platform = 35
build-tools = "35.0.0"

[target.android]
rustflags = ["-C", "link-arg=-landroid"]
```

- **identifier**: Android package name
- **platform = 35**: Android 15 (API level 35)
- **rustflags**: Links Android native activity library

---

## epidemic-core Crate

### Dependencies (Cargo.toml)
```toml
[dependencies]
resvg = "0.45"          # SVG rendering engine
usvg = "0.45"           # SVG parsing (used by resvg)
tiny-skia = "0.11"      # 2D rasterizer (used for path filling)
tiny-skia-path = "0.11" # Path data types
```

### Module: lib.rs (11 lines)
Exports all public types:
```rust
pub mod disease;     // Disease model
pub mod map_data;    // Old grid map (unused)
pub mod regions;     // Region definitions
pub mod sim;         // GameState stub
pub mod svg_parser;  // SVG rasterizer
pub mod world;       // World simulation

pub use disease::{Disease, PathogenType, Upgrade, UpgradeCategory, all_upgrades};
pub use regions::Region;
pub use sim::GameState;
pub use world::{Difficulty, GamePhase, World};
```

---

### Module: disease.rs (379 lines)

#### Enum: PathogenType (lines 3-12)
```rust
pub enum PathogenType {
    Bacteria,    // Standard, cheap devolve
    Virus,       // Random mutations
    Fungus,      // Slow spread, spore launch
    Parasite,    // Low severity stealth
    Prion,       // Slow infection, slows cure
    NanoVirus,   // Cure starts immediately
    BioWeapon,   // Innate lethality
}
```

**Methods:**
- `name(&self) -> &str` (line 15): Returns display name
- `description(&self) -> &str` (line 27): Returns gameplay description
- `base_infectivity(&self) -> f32` (line 39): Base infection rate multiplier
  - Bacteria: 1.0, Virus: 1.1, Fungus: 0.5, Parasite: 0.8, Prion: 0.6, NanoVirus: 1.2, BioWeapon: 1.0
- `base_severity(&self) -> f32` (line 51): Base severity (triggers cure research)
  - Bacteria: 1.0, Virus: 1.2, Fungus: 0.8, Parasite: 0.3, Prion: 0.7, NanoVirus: 1.5, BioWeapon: 1.0
- `base_lethality(&self) -> f32` (line 63): Base death rate
  - All: 0.0 except BioWeapon: 0.1

#### Enum: UpgradeCategory (lines 76-81)
```rust
pub enum UpgradeCategory {
    Transmission,  // How it spreads
    Symptom,       // What it does
    Ability,       // Passive buffs
}
```

#### Struct: Upgrade (lines 83-94)
```rust
pub struct Upgrade {
    pub id: &'static str,           // Unique identifier (e.g., "air1")
    pub name: &'static str,         // Display name (e.g., "Air 1")
    pub category: UpgradeCategory,  // Transmission/Symptom/Ability
    pub cost: u32,                  // DNA point cost
    pub infectivity: f32,           // Added infectivity
    pub severity: f32,              // Added severity
    pub lethality: f32,             // Added lethality
    pub requires: Vec<&'static str>, // Prerequisites (upgrade IDs)
    pub description: &'static str,  // Tooltip text
}
```

#### Function: all_upgrades() (lines 96-316)
Returns all 35 upgrades:

**Transmission (12 upgrades):**
| ID | Name | Cost | Infectivity | Severity | Requires |
|----|------|------|-------------|----------|----------|
| air1 | Air 1 | 9 | +3.0 | 0.0 | — |
| air2 | Air 2 | 13 | +7.0 | 0.0 | air1 |
| water1 | Water 1 | 9 | +3.0 | 0.0 | — |
| water2 | Water 2 | 13 | +7.0 | 0.0 | water1 |
| insect1 | Insect 1 | 9 | +5.0 | +1.0 | — |
| insect2 | Insect 2 | 14 | +10.0 | +1.0 | insect1 |
| bird1 | Bird 1 | 10 | +4.0 | 0.0 | — |
| bird2 | Bird 2 | 15 | +7.0 | 0.0 | bird1 |
| blood1 | Blood 1 | 10 | +5.0 | +2.0 | — |
| blood2 | Blood 2 | 15 | +10.0 | +2.0 | blood1 |
| rodent1 | Rodent 1 | 8 | +4.0 | 0.0 | — |
| rodent2 | Rodent 2 | 12 | +8.0 | 0.0 | rodent1 |

**Symptoms (19 upgrades, 4 tiers):**

Tier 1 — Mild:
| ID | Name | Cost | Infectivity | Severity | Lethality | Requires |
|----|------|------|-------------|----------|-----------|----------|
| coughing | Coughing | 3 | +2.0 | +1.0 | 0.0 | — |
| nausea | Nausea | 3 | +1.0 | +1.0 | 0.0 | — |
| rash | Rash | 3 | +1.0 | +1.0 | 0.0 | — |
| insomnia | Insomnia | 3 | 0.0 | +1.0 | 0.0 | — |
| cysts | Cysts | 4 | +1.0 | +2.0 | 0.0 | — |

Tier 2 — Moderate:
| ID | Name | Cost | Infectivity | Severity | Lethality | Requires |
|----|------|------|-------------|----------|-----------|----------|
| pneumonia | Pneumonia | 6 | +3.0 | +2.0 | +1.0 | coughing |
| vomiting | Vomiting | 5 | +3.0 | +2.0 | 0.0 | nausea |
| sweating | Sweating | 5 | +1.0 | +1.0 | 0.0 | rash |
| paranoia | Paranoia | 6 | 0.0 | +2.0 | 0.0 | insomnia |
| abscesses | Abscesses | 5 | +1.0 | +3.0 | 0.0 | cysts |

Tier 3 — Severe:
| ID | Name | Cost | Infectivity | Severity | Lethality | Requires |
|----|------|------|-------------|----------|-----------|----------|
| pulmonary_fibrosis | Pulmonary Fibrosis | 10 | +2.0 | +3.0 | +3.0 | pneumonia |
| diarrhea | Diarrhea | 7 | +5.0 | +2.0 | +1.0 | vomiting |
| skin_lesions | Skin Lesions | 8 | +4.0 | +4.0 | +1.0 | sweating |
| seizures | Seizures | 8 | +1.0 | +4.0 | +2.0 | paranoia |
| necrosis | Necrosis | 12 | +2.0 | +5.0 | +5.0 | abscesses |

Tier 4 — Lethal:
| ID | Name | Cost | Infectivity | Severity | Lethality | Requires |
|----|------|------|-------------|----------|-----------|----------|
| total_organ_failure | Total Organ Failure | 18 | 0.0 | +8.0 | +12.0 | pulmonary_fibrosis |
| hemorrhagic_shock | Hemorrhagic Shock | 15 | 0.0 | +6.0 | +10.0 | diarrhea |
| coma | Coma | 14 | 0.0 | +7.0 | +8.0 | seizures |
| immune_suppression | Immune Suppression | 12 | +3.0 | +5.0 | +4.0 | necrosis |
| dysentery | Dysentery | 10 | +4.0 | +4.0 | +6.0 | diarrhea |

**Abilities (9 upgrades):**
| ID | Name | Cost | Effect | Requires |
|----|------|------|--------|----------|
| drug_resistance1 | Drug Resistance 1 | 12 | +50% infection in wealthy countries | — |
| drug_resistance2 | Drug Resistance 2 | 18 | +100% infection in wealthy countries | drug_resistance1 |
| cold_resistance1 | Cold Resistance 1 | 10 | +50% infection in cold climates | — |
| cold_resistance2 | Cold Resistance 2 | 15 | +100% infection in cold climates | cold_resistance1 |
| heat_resistance1 | Heat Resistance 1 | 10 | +50% infection in hot climates | — |
| heat_resistance2 | Heat Resistance 2 | 15 | +100% infection in hot climates | heat_resistance1 |
| genetic_hardening1 | Genetic Hardening 1 | 15 | Cure research speed -5% | — |
| genetic_hardening2 | Genetic Hardening 2 | 20 | Cure research speed -10% | genetic_hardening1 |
| genetic_reshuffle1 | Genetic Reshuffle | 30 | Resets cure progress by 25% | genetic_hardening2 |

#### Struct: Disease (lines 318-327)
```rust
pub struct Disease {
    pub name: String,                    // Player-chosen name
    pub pathogen_type: PathogenType,     // Bacteria/Virus/etc.
    pub upgrades: HashMap<String, bool>, // id -> unlocked
    pub total_infectivity: f32,          // Sum of base + upgrades
    pub total_severity: f32,             // Sum of base + upgrades
    pub total_lethality: f32,            // Sum of base + upgrades
    pub cure_slowdown: f32,              // % reduction in cure speed
}
```

**Methods:**
- `new(name, pathogen_type)` (line 330): Creates disease with base stats
- `has_upgrade(id) -> bool` (line 342): Check if upgrade is owned
- `can_unlock(upgrade) -> bool` (line 346): Check prerequisites met
- `unlock(upgrade)` (line 353): Purchase upgrade, add stats
- `effective_infectivity() -> f32` (line 368): Clamped >= 0
- `effective_severity() -> f32` (line 372): Clamped >= 0
- `effective_lethality() -> f32` (line 376): Clamped >= 0

---

### Module: world.rs (579 lines)

#### Struct: World (lines 5-26)
```rust
pub struct World {
    pub regions: Vec<Region>,         // 64 regions
    pub svg_lookup: Vec<u16>,         // 800x400 pixel -> region ID
    pub lookup_w: usize,              // 800
    pub lookup_h: usize,              // 400
    pub tick: u64,                    // Current simulation tick
    pub game_speed: u32,              // 1x, 2x, 3x
    pub dna_points: u32,              // Available DNA points
    pub total_infected: u64,          // Global infected count
    pub total_dead: u64,              // Global dead count
    pub total_healthy: u64,           // Global healthy count
    pub cure_progress: f32,           // 0.0 to 100.0
    pub news: Vec<String>,            // News headlines (max 5)
    pub phase: GamePhase,             // Current game phase
    pub selected_region: Option<u16>, // Outbreak origin
    pub disease: Disease,             // Active disease
    pub upgrades: Vec<Upgrade>,       // All available upgrades
    pub events: Vec<GameEvent>,       // Event history
    pub dna_bubbles: Vec<DnaBubble>,  // Collectible DNA bubbles
    pub difficulty: Difficulty,        // Casual/Normal/Brutal/MegaBrutal
    pub disease_name: String,         // Disease display name
}
```

#### Enum: Difficulty (lines 28-63)
```rust
pub enum Difficulty {
    Casual,      // cure_speed: 0.5x, border_close: 0.5x
    Normal,      // cure_speed: 1.0x, border_close: 1.0x
    Brutal,      // cure_speed: 1.5x, border_close: 1.5x
    MegaBrutal,  // cure_speed: 2.0x, border_close: 2.0x
}
```

#### Enum: GamePhase (lines 92-101)
```rust
pub enum GamePhase {
    TitleScreen,       // Logo + New Game button
    PathogenSelect,    // 7 disease type cards
    DifficultySelect,  // 4 difficulty options
    SelectOrigin,      // Click country on map
    Playing,           // Simulation running
    Won,               // All humans dead
    Lost,              // Cure at 100%
}
```

#### Struct: GameEvent (lines 65-81)
```rust
pub struct GameEvent {
    pub tick: u64,
    pub message: String,
    pub event_type: EventType,
}

pub enum EventType {
    NewCountry(String),        // Infection reached new country
    BorderClosed(String),      // Country closed borders
    CureMilestone(f32),        // Cure at 10/25/50/75/90%
    SportsEvent(String),       // Infection spike from mass gathering
    InfectedPlane(String, String), // Air transmission event
    InfectedShip(String, String),  // Sea transmission event
    ResearchBoost,             // Cure research accelerated
}
```

#### Struct: DnaBubble (lines 83-90)
```rust
pub struct DnaBubble {
    pub x: f32,            // 0.0-1.0 normalized position
    pub y: f32,            // 0.0-1.0 normalized position
    pub value: u32,        // DNA points awarded
    pub tick_spawned: u64, // When it appeared
    pub collected: bool,   // Already collected?
}
```

#### Methods on World

**new(svg_content) -> Self** (line 105)
1. Calls `regions::build_regions()` to create 64 regions
2. Builds SVG code -> region ID map
3. Rasterizes SVG to 800x400 lookup table via `svg_parser::rasterize_svg_to_lookup()`
4. Maps raw SVG country IDs to region IDs
5. Initializes all fields to defaults

**init_disease(name, pathogen_type)** (line 158)
Creates a new Disease with the given name and type.

**start_outbreak(region_id)** (line 164)
Sets `infected = 1` for the region, pushes news, sets phase to Playing.

**region_at_pixel(px, py) -> Option<&Region>** (line 175)
Looks up region ID in the SVG lookup table at pixel coordinates.

**advance()** (line 187) — Main simulation tick:
1. Increments tick
2. Reads disease stats (infectivity, severity, lethality)
3. Earns DNA points passively (every 50 ticks)
4. Spawns DNA bubbles (every 30 ticks, max 5)
5. Expires old bubbles (200 tick lifetime)
6. Calculates infections per region:
   - `new_infected = infected * 0.0008 * infectivity * drug_resistance * (healthy / population)`
   - Wealthy countries get 0.5x resistance
   - Drug Resistance upgrades counter this
7. Calculates deaths per region:
   - `new_deaths = infected * 0.00002 * lethality`
8. Applies infections (triggers NewCountry events)
9. Applies deaths (triggers fallen status)
10. Calls `cross_border_spread()`
11. Calls `update_borders(severity)`
12. Calls `update_cure(severity)`
13. Calls `random_events()`
14. Recounts totals
15. Checks endgame

**cross_border_spread()** (line 304)
- For each infected region with >1% infection:
- Checks neighbor pairs
- Chance of seeding infection in uninfected neighbor: `infection_pct * 0.001`

**get_neighbor_pairs() -> Vec<(u16, u16)>** (line 347)
Returns hardcoded adjacency list:
- North America: US-CA, US-MX, MX-CAM
- South America: BR-AR, BR-CO, BR-SA, CO-PE, PE-SA
- Europe: GB-FR, FR-DE, DE-ES, DE-PT, ES-IT, FR-IT, DE-WE, DE-NE, DE-PL, PL-UA, UA-EE, DE-EE
- Russia: DE-RU, NE-RU, PL-RU, EE-RU
- Middle East: RU-TR, TR-SA2, SA2-IR, IR-IQ, IQ-ME
- Africa: ME-EG, EG-DZ, DZ-MA, MA-NA, NA-NG, NG-GH, GH-WA, EG-ET, ET-KE, KE-TZ, TZ-EA, WA-CD, CD-CF, CF-ZA, ZA-SA3
- Asia: RU-KZ, KZ-CA2, CA2-IN, IN-PK, PK-BD, BD-SA4, SA4-ID, ID-TH, TH-VN, VN-PH, PH-MM, MM-MY, MY-SEA, RU-CN, CN-JP, CN-KR, KR-KP, CN-TW, ID-AU, SEA-OC, ID-OC
- Oceania: AU-NZ, AU-OC

**update_cure(severity)** (line 385)
- Starts when 3+ countries infected AND severity > 0.5
- Per-region research speed: wealthy=0.008, others=0.003
- Scaled by severity factor (severity/10, max 2.0)
- Reduced by dead researchers (dead_pct * 0.5, min 0.3)
- Modified by disease cure_slowdown
- Milestones at 10%, 25%, 50%, 75%, 90%

**update_borders(severity)** (line 428)
- Requires severity >= 1.0
- For each neighbor pair where one is infected and one is healthy:
- Close chance = severity / 50 (max 0.1)
- Modified by difficulty border_close_mult

**random_events()** (line 465)
- Every 100 ticks, roll random:
- 0.0-0.15: Sports event (infection spike in random infected country)
- 0.15-0.25: Research funding boost (cure +2.0)

**collect_bubble(x, y) -> bool** (line 502)
Checks if click position (normalized 0-1) is within 0.03 of any uncollected bubble.

**recount()** (line 518)
Recalculates total_infected, total_dead, total_healthy from all regions.

**check_endgame()** (line 529)
- Win: no healthy people left anywhere
- Lose: cure_progress >= 100.0

**render_data() -> Vec<f32>** (line 544)
Returns GPU-ready data: [u, v, region_id, infection_pct] per pixel.

#### Function: pseudo_rand(tick, a, b) -> f32 (line 572)
Deterministic hash-based pseudo-random number generator. Returns 0.0-1.0.

---

### Module: regions.rs (187 lines)

#### Struct: Region (lines 2-14)
```rust
pub struct Region {
    pub id: u16,              // Matches SVG lookup table
    pub code: String,         // ISO code (e.g., "US")
    pub name: String,         // Display name (e.g., "United States")
    pub population: u64,      // 2026 estimate
    pub infected: u64,        // Currently infected
    pub dead: u64,            // Dead count
    pub borders_open: bool,   // Can infection spread in/out?
    pub cure_progress: f32,   // 0.0 to 100.0
    pub fallen: bool,         // All dead?
    pub svg_codes: Vec<String>, // All SVG country codes in this region
}
```

**Methods:**
- `new(id, code, name, population, svg_codes)` (line 17)
- `healthy() -> u64` (line 32): `population - infected - dead`
- `infection_pct() -> f32` (line 36): `infected / population`
- `death_pct() -> f32` (line 43): `dead / population`

#### Function: build_regions() -> Vec<Region> (lines 53-175)
Returns 64 regions with 2026 population data:

| ID | Code | Name | Population | SVG Codes |
|----|------|------|------------|-----------|
| 1 | US | United States | 341,800,000 | US |
| 2 | CA | Canada | 41,000,000 | CA |
| 3 | MX | Mexico | 130,000,000 | MX |
| 4 | CAM | Central America | 55,000,000 | GT,BZ,HN,SV,NI,CR,PA,CU,JM,HT,DO,TT,BB,GD,LC,VC,AG,KN,DM,BS,PR,VI,AI,AW,CW,SX,MF,BL,PM |
| 5 | BR | Brazil | 216,000,000 | BR |
| 6 | AR | Argentina | 47,000,000 | AR |
| 7 | CO | Colombia | 52,000,000 | CO |
| 8 | PE | Peru | 34,000,000 | PE |
| 9 | VE | Venezuela | 28,000,000 | VE |
| 10 | SA | South America Rest | 45,000,000 | CL,EC,BO,PR,UY,GY,SR,GF |
| 11 | GB | United Kingdom | 69,000,000 | GB |
| 12 | FR | France | 68,000,000 | FR |
| 13 | DE | Germany | 84,000,000 | DE |
| 14 | ES | Spain | 48,000,000 | ES |
| 15 | PT | Portugal | 10,400,000 | PT |
| 16 | IT | Italy | 59,000,000 | IT |
| 17 | WE | Western Europe | 75,000,000 | NL,BE,LU,CH,AT,IE |
| 18 | NE | Northern Europe | 28,000,000 | SE,NO,DK,FI,IS |
| 19 | PL | Poland | 38,000,000 | PL |
| 20 | UA | Ukraine | 37,000,000 | UA |
| 21 | EE | Eastern Europe | 85,000,000 | CZ,SK,HU,RO,BG,HR,RS,BA,ME,MK,AL,XK,SI,EE,LV,LT,MD,BY,GE,AM,AZ |
| 22 | RU | Russia | 144,000,000 | RU |
| 23 | TR | Turkey | 86,000,000 | TR |
| 24 | SA2 | Saudi Arabia | 37,000,000 | SA |
| 25 | IR | Iran | 88,000,000 | IR |
| 26 | IQ | Iraq | 43,000,000 | IQ |
| 27 | ME | Middle East Rest | 65,000,000 | AE,IL,JO,LB,SY,YE,OM,QA,BH,KW,PS |
| 28 | EG | Egypt | 106,000,000 | EG |
| 29 | DZ | Algeria | 46,000,000 | DZ |
| 30 | MA | Morocco | 37,500,000 | MA |
| 31 | NA | North Africa Rest | 45,000,000 | TN,LY,SD,SS,EH |
| 32 | NG | Nigeria | 224,000,000 | NG |
| 33 | GH | Ghana | 34,000,000 | GH |
| 34 | WA | West Africa Rest | 180,000,000 | SN,ML,BF,NE,CI,GN,SL,LR,BJ,TG,MR,GM,GW,GN,CV |
| 35 | ET | Ethiopia | 126,000,000 | ET |
| 36 | KE | Kenya | 56,000,000 | KE |
| 37 | TZ | Tanzania | 65,000,000 | TZ |
| 38 | EA | East Africa Rest | 140,000,000 | UG,RW,BI,DJ,ER,SO,MG,MZ,MW,ZM,ZW |
| 39 | CD | DR Congo | 102,000,000 | CD |
| 40 | CF | Central Africa Rest | 70,000,000 | CM,CG,GA,GQ,TD,CF,AO,ST |
| 41 | ZA | South Africa | 62,000,000 | ZA |
| 42 | SA3 | Southern Africa Rest | 30,000,000 | NA,BW,SZ,LS |
| 43 | KZ | Kazakhstan | 20,000,000 | KZ |
| 44 | CA2 | Central Asia Rest | 60,000,000 | UZ,TM,KG,TJ,AF,MN |
| 45 | IN | India | 1,450,000,000 | IN |
| 46 | PK | Pakistan | 240,000,000 | PK |
| 47 | BD | Bangladesh | 175,000,000 | BD |
| 48 | SA4 | South Asia Rest | 55,000,000 | NP,LK,BT,MV,AF |
| 49 | ID | Indonesia | 280,000,000 | ID |
| 50 | TH | Thailand | 72,000,000 | TH |
| 51 | VN | Vietnam | 100,000,000 | VN |
| 52 | PH | Philippines | 117,000,000 | PH |
| 53 | MM | Myanmar | 55,000,000 | MM |
| 54 | MY | Malaysia | 34,000,000 | MY |
| 55 | SEA | Southeast Asia Rest | 60,000,000 | KH,LA,BN,TL,SG,BN |
| 56 | CN | China | 1,425,000,000 | CN |
| 57 | JP | Japan | 124,000,000 | JP |
| 58 | KR | South Korea | 52,000,000 | KR |
| 59 | KP | North Korea | 26,000,000 | KP |
| 60 | TW | Taiwan | 24,000,000 | TW |
| 61 | AU | Australia | 27,000,000 | AU |
| 62 | NZ | New Zealand | 5,200,000 | NZ |
| 63 | OC | Oceania Rest | 15,000,000 | PG,FJ,SB,VU,WS,TO,KI,MH,FM,PW,TV,NR,NC,PF,GU |
| 64 | GL | Greenland | 57,000 | GL |

#### Function: svg_code_to_region_map(regions) -> HashMap<String, u16> (line 179)
Creates a mapping from SVG country code to region ID.

---

### Module: svg_parser.rs (210 lines)

#### Struct: SvgCountry (lines 5-9)
```rust
pub struct SvgCountry {
    pub code: String,  // Country code (e.g., "US")
    pub name: String,  // Display name (e.g., "United States")
}
```

#### Function: preprocess_svg(svg) -> String (lines 11-25)
Converts `class="Country Name"` to `id="CODE"` in SVG paths that lack an `id` attribute. This ensures all countries have IDs for the rasterizer.

#### Function: parse_world_svg(svg_content) -> Vec<SvgCountry> (lines 27-46)
Extracts all country metadata from SVG `<path>` elements. Deduplicates by code.

#### Function: extract_attr(line, attr) -> Option<String> (lines 48-57)
Simple XML attribute extraction from a single line.

#### Function: class_name_to_code(class) -> String (lines 59-120)
Maps full country names to ISO codes. Handles:
- Full names: "United Kingdom" -> "GB", "United States" -> "US"
- Special cases: "Dem. Rep. Korea" -> "KP", "Côte d'Ivoire" -> "CI"
- Fallback: Takes first 2 alphabetic characters, uppercase

#### Function: rasterize_svg_to_lookup(svg_content, width, height) -> (Vec<u16>, Vec<SvgCountry>) (lines 122-150)
1. Preprocesses SVG (class -> id)
2. Parses countries
3. Builds code -> index map (1-based, 0 = ocean)
4. Parses SVG with usvg
5. Walks the SVG tree, rendering each country path to a lookup table
6. Returns (lookup_table, countries_list)

#### Function: render_group(group, code_to_idx, lookup, width, height, sx, sy) (lines 152-178)
Recursively walks the usvg tree. For each Path node with a matching country ID, fills it into the lookup.

#### Function: fill_path(path, lookup, width, height, country_id, sx, sy) (lines 180-210)
1. Creates a temporary Pixmap
2. Renders the path with white color at the correct scale
3. Reads back pixels — any white pixel in the pixmap sets the lookup table entry

---

### Module: sim.rs (31 lines)

#### Struct: GameState
```rust
pub struct GameState {
    pub tick: u64,
    pub paused: bool,
}
```
Simple tick counter with pause. Used for the old grid system, largely superseded by World.

---

### Module: map_data.rs (unused)
Old grid-based world map. No longer used — replaced by SVG-based rendering.

---

## epidemic-render Crate

### Dependencies (Cargo.toml)
```toml
wgpu = { version = "24", features = ["wgsl"] }
winit = { version = "0.30", features = ["rwh_06"] }
pollster = "0.4"
bytemuck = { version = "1", features = ["derive"] }
epidemic-core = { path = "../epidemic-core" }
log = "0.4"
egui = "0.31"
egui-wgpu = "0.31"
egui-winit = "0.31"
image = "0.25"
```

### Module: lib.rs (1072 lines)

#### Struct: Uniforms (lines 15-22)
```rust
struct Uniforms {
    time: f32,    // Elapsed seconds (for animations)
    map_w: f32,   // Map width in pixels (800)
    map_h: f32,   // Map height in pixels (400)
    _pad: f32,    // Alignment padding
}
```

#### Struct: Renderer (lines 24-40)
```rust
pub struct Renderer {
    surface: wgpu::Surface<'static>,      // Window surface
    device: wgpu::Device,                 // GPU device
    queue: wgpu::Queue,                   // Command queue
    config: wgpu::SurfaceConfiguration,   // Surface format/size
    size: PhysicalSize<u32>,              // Window size
    pipeline: wgpu::RenderPipeline,       // Map render pipeline
    uniform_buffer: wgpu::Buffer,         // Uniform data
    bind_group: wgpu::BindGroup,          // Resource bindings
    start_time: Instant,                  // For time uniform
    map_texture: wgpu::Texture,           // 800x400 country texture
    logo_texture: Option<egui::TextureHandle>, // Logo loaded as egui texture
    egui_ctx: egui::Context,              // egui context
    egui_state: egui_winit::State,        // egui-winit bridge
    egui_renderer: egui_wgpu::Renderer,   // egui-wgpu renderer
}
```

#### Methods on Renderer

**async new(window, world) -> Self** (line 43)
1. Creates wgpu Instance with all backends
2. Creates Surface from window
3. Requests Adapter (GPU)
4. Requests Device + Queue
5. Configures Surface (format, size, vsync)
6. Creates map Texture (800x400, RGBA8)
7. Creates Sampler (nearest-neighbor)
8. Creates Uniforms buffer
9. Creates BindGroupLayout (uniform + texture + sampler)
10. Creates BindGroup
11. Loads map.wgsl shader
12. Creates RenderPipeline (fullscreen quad)
13. Sets up egui (Context, State, Renderer)

**resize(new_size)** (line 249)
Updates surface config when window resizes.

**handle_event(window, event) -> bool** (line 258)
Passes events to egui. Returns false for mouse clicks (so game can handle them).

**render(world, window, hovered_region) -> Result** (line 267)
1. Loads logo on first frame (via `load_logo()`)
2. Updates uniforms (time, map dimensions)
3. Builds map texture CPU-side via `build_map_texture()`
4. Uploads texture to GPU
5. Runs egui UI via `build_ui()`
6. Tessellates egui output
7. Creates command encoder
8. Pass 1: Renders map (fullscreen quad with texture)
9. Pass 2: Renders egui HUD (alpha-blended on top)
10. Submits commands, presents frame

**screen_to_map(pos, world) -> (usize, usize)** (line 421)
Converts screen pixel position to map pixel coordinates.

---

#### Function: build_map_texture(world, hovered_region) -> Vec<u8> (lines 430-525)
Builds the 800x400 RGBA texture on CPU every frame.

**Border detection** (lines 436-458):
For each land pixel, checks 4 neighbors (N/S/E/W). If any neighbor has a different region ID, marks as border.

**Color logic** (lines 461-522):
- Ocean (region_id == 0): RGB(5, 10, 25) — dark blue
- Border pixel: RGB(15, 15, 20) — dark outline
- Fallen region: RGB(30, 30, 30) — dark gray
- Healthy region: RGB(20, 70, 30) — green
  - If hovered: add +60 to each channel (white tint)
- Infected region: blend green -> red based on infection %
  - R = 20 + pct * 200
  - G = 70 * (1 - pct)
  - B = 30 * (1 - pct)
  - If hovered: add +50 to each channel

---

#### Function: build_ui(ctx, world, logo) (lines 527-557)
Routes to the correct UI based on game phase:
- `TitleScreen` -> `build_title_screen()`
- `PathogenSelect` -> `build_pathogen_select()`
- `DifficultySelect` -> `build_difficulty_select()`
- `SelectOrigin/Playing/Won/Lost` -> `build_gameplay_hud()`

**Color palette (Moo UI dark theme):**
- bg: RGB(33, 37, 41)
- surface: RGB(52, 58, 64)
- border: RGB(73, 80, 87)
- text: RGB(222, 226, 230)
- muted: RGB(113, 113, 122)
- heading: RGB(255, 255, 255)
- success: RGB(4, 120, 87)
- danger: RGB(231, 0, 11)
- info: RGB(3, 105, 161)
- warning: RGB(180, 83, 9)

---

#### Function: build_title_screen() (lines 559-597)
- CentralPanel with dark background
- Logo image (or fallback text)
- "NEW GAME" button (200x48, info blue)
- Version label

#### Function: build_pathogen_select() (lines 599-657)
- CentralPanel
- "SELECT PATHOGEN" heading
- 2-column grid of 7 pathogen cards
- Each card: name, description, colored SELECT button
- Clicking sets disease and moves to DifficultySelect

#### Function: build_difficulty_select() (lines 659-711)
- CentralPanel
- "SELECT DIFFICULTY" heading
- Shows selected pathogen name
- 4 difficulty cards (Casual/Normal/Brutal/MegaBrutal)
- Each card: name, description, colored SELECT button
- Clicking sets difficulty and moves to SelectOrigin

#### Function: build_gameplay_hud() (lines 713-866)

**Left Panel (240px):**
1. "EPIDEMIC NS" title
2. Tick counter + speed indicator
3. Population card: Healthy (green), Infected (red), Dead (gray)
4. DNA Points card: Large number in info blue
5. Cure Progress card: Progress bar + percentage
6. Disease card: Infectivity, Severity, Lethality stats
7. Speed buttons: 1x/2x/3x (active = filled info blue)
8. Phase indicator: SelectOrigin (warning), Playing (danger), Won (success), Lost (info)

**Bottom Panel (36px):**
- News ticker showing latest headline in warning amber
- "No active reports" in muted when empty

**Right Panel (260px, during Playing only):**
- "EVOLUTION" title
- DNA points display
- Collapsible sections:
  - Transmission (success green)
  - Symptoms (warning amber)
  - Abilities (info blue)
- Each upgrade: checkmark if owned, button if affordable, grayed if locked/affordable

---

#### Function: stat_row(ui, label, value, color) (lines 868-875)
Helper: renders a label-value row with the value right-aligned.

#### Function: load_logo() -> Result<RgbaImage> (lines 877-883)
Tries multiple paths to load EPIDEMIC.png:
- `../assets/EPIDEMIC.png`
- `assets/EPIDEMIC.png`
- `../Assets/EPIDEMIC.png`
- `Assets/EPIDEMIC.png`

#### Function: format_num(n) -> String (lines 885-895)
Formats large numbers: 1,000 -> "1.0K", 1,000,000 -> "1.0M", 1,000,000,000 -> "1.0B"

---

#### Struct: App (lines 897-905)
```rust
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    last_sim_tick: Instant,
    sim_interval_ms: u64,
    cursor_pos: PhysicalPosition<f64>,
    hovered_region: Option<u16>,
}
```

**BASE_SIM_INTERVAL = 60ms** (~16.7 ticks/sec at 1x)

#### App::new() (lines 910-928)
1. Loads world.svg from assets
2. Creates World
3. Initializes disease as Bacteria with name "Epidemic"

#### ApplicationHandler implementation (lines 931-1065)

**resumed()** (line 932)
Creates window (1280x720) and Renderer on first call.

**window_event()** (line 945)

- **CloseRequested**: exit
- **Resized**: resize renderer
- **CursorMoved**: update cursor_pos, compute hovered_region
- **MouseInput (Left Pressed)**: If in SelectOrigin, start outbreak at hovered country
- **KeyboardInput**:
  - Space: toggle pause
  - Escape: exit
  - Digit1: speed 1x
  - Digit2: speed 2x
  - Digit3: speed 3x
- **RedrawRequested**:
  1. Sync sim_interval with game_speed
  2. If Playing and not paused and enough time elapsed: call world.advance(), trim news
  3. Render frame
  4. Request next redraw (continuous)

---

## Shaders

### map.wgsl (55 lines) — ACTIVE
Full-screen quad shader for the world map.

**Bindings:**
- @group(0) @binding(0): Uniforms (time, map_w, map_h)
- @group(0) @binding(1): texture_2d<f32> (map texture)
- @group(0) @binding(2): sampler

**Vertex shader (vs_main):**
Generates 6 vertices for 2 triangles forming a fullscreen quad. UVs map texture to screen.

**Fragment shader (fs_main):**
Samples map texture at interpolated UV. Returns color directly.

### grid.wgsl (85 lines) — UNUSED
Old instanced grid shader. Kept for reference.

### triangle.wgsl (19 lines) — UNUSED
Old colored triangle shader. Kept for reference.

---

## Assets

### world.svg (977 lines)
MIT-licensed world map from Simplemaps.com. Contains:
- 196 country paths with `id` (ISO codes) or `class` (full names)
- ViewBox: 0 0 2000 857
- Some countries have multiple paths (islands, territories)
- Country identification: `id="BR"` or `class="Brazil"`

### EPIDEMIC.png
Game logo for title screen.

### plane737.svg, ship.svg, biohazard.svg
Game icons (not yet integrated into rendering).

---

## Game Flow

```
App::new()
  └─> World::new(svg) — loads map, creates regions
  └─> init_disease("Epidemic", Bacteria)
  └─> phase = TitleScreen

TitleScreen
  └─> Click "NEW GAME"
  └─> phase = PathogenSelect

PathogenSelect
  └─> Click pathogen card
  └─> init_disease(name, type)
  └─> phase = DifficultySelect

DifficultySelect
  └─> Click difficulty
  └─> world.difficulty = diff
  └─> phase = SelectOrigin

SelectOrigin
  └─> Click country on map
  └─> start_outbreak(region_id)
  └─> phase = Playing

Playing
  └─> advance() every 60ms (1x) / 30ms (2x) / 15ms (3x)
  └─> Collect DNA bubbles
  └─> Purchase upgrades
  └─> Win: all healthy == 0 → phase = Won
  └─> Lose: cure >= 100% → phase = Lost
```

---

## Simulation Model

### Infection Spread (per region, per tick)
```
new_infected = infected * 0.0008 * infectivity * drug_resistance * drug_bonus * (healthy / population)
```
- infectivity: sum of base + all upgrade values
- drug_resistance: 0.5 for wealthy countries (US, GB, DE, FR, JP, KR, AU, CA, IT, ES, NL, SE, CH, WE, NE)
- drug_bonus: 1.5 with Drug Resistance 1, 2.0 with Drug Resistance 2 (only for wealthy)

### Death Rate (per region, per tick)
```
new_deaths = infected * 0.00002 * lethality
```

### Cross-Border Spread
- Requires source region > 1% infected
- Requires target region borders_open
- Chance = infection_pct * 0.001

### Cure Research
- Starts when 3+ countries infected AND severity > 0.5
- Per region: wealthy = 0.008/tick, others = 0.003/tick
- Scaled by severity factor (severity/10, max 2.0)
- Reduced by dead researchers
- Modified by disease cure_slowdown

### Border Closure
- Requires severity >= 1.0
- For infected-healthy neighbor pairs
- Close chance = severity/50 (max 0.1)

### DNA Points
- Passive: 1 + min(total_infected/10M, 5) per 50 ticks
- Bubbles: 1 + severity/3 per bubble, spawned every 30 ticks (max 5)
- Bubble expires after 200 ticks

### Events
- Every 100 ticks:
  - 15% chance: Sports event (infection spike)
  - 10% chance: Research funding boost
