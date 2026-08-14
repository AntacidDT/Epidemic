pub mod disease;
pub mod map_data;
pub mod regions;
pub mod save;
pub mod sim;
pub mod svg_parser;
pub mod world;

pub use disease::{Disease, PathogenType, Upgrade, UpgradeCategory, all_upgrades};
pub use regions::{Region, Climate, Density, GovernmentType};
pub use save::{GameScore, SaveData, calculate_score, load_game, save_game};
pub use sim::GameState;
pub use world::{
    CurePhase, Difficulty, DnaBubble, EventType, GameEvent, GamePhase, GameType,
    Season, Synergy, TacticalAbility, TransportEntity, TransportType, World,
};
