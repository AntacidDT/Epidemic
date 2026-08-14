use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathogenType {
    Bacteria,
    Virus,
    Fungus,
    Parasite,
    Prion,
    NanoVirus,
    BioWeapon,
}

impl PathogenType {
    pub fn name(&self) -> &str {
        match self {
            Self::Bacteria => "Bacteria",
            Self::Virus => "Virus",
            Self::Fungus => "Fungus",
            Self::Parasite => "Parasite",
            Self::Prion => "Prion",
            Self::NanoVirus => "Nano-Virus",
            Self::BioWeapon => "Bio-Weapon",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Bacteria => "Standard pathogen. Cheap to devolve, fast mutation.",
            Self::Virus => "Random mutations appear for free. Uncontrollable.",
            Self::Fungus => "Very slow spread. Can launch spores to specific countries.",
            Self::Parasite => "Low severity stealth pathogen. Symbiosis suppresses symptoms.",
            Self::Prion => "Slow infection. Slows cure research.",
            Self::NanoVirus => "Cure research starts immediately. Must race.",
            Self::BioWeapon => "Kills automatically. Must suppress with gene compression.",
        }
    }

    pub fn base_infectivity(&self) -> f32 {
        match self {
            Self::Bacteria => 1.0,
            Self::Virus => 1.1,
            Self::Fungus => 0.5,
            Self::Parasite => 0.8,
            Self::Prion => 0.6,
            Self::NanoVirus => 1.2,
            Self::BioWeapon => 1.0,
        }
    }

    pub fn base_severity(&self) -> f32 {
        match self {
            Self::Bacteria => 1.0,
            Self::Virus => 1.2,
            Self::Fungus => 0.8,
            Self::Parasite => 0.3,
            Self::Prion => 0.7,
            Self::NanoVirus => 1.5,
            Self::BioWeapon => 1.0,
        }
    }

    pub fn base_lethality(&self) -> f32 {
        match self {
            Self::Bacteria => 0.0,
            Self::Virus => 0.0,
            Self::Fungus => 0.0,
            Self::Parasite => 0.0,
            Self::Prion => 0.0,
            Self::NanoVirus => 0.0,
            Self::BioWeapon => 0.1, // innate lethality
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpgradeCategory {
    Transmission,
    Symptom,
    Ability,
}

#[derive(Debug, Clone)]
pub struct Upgrade {
    pub id: &'static str,
    pub name: &'static str,
    pub category: UpgradeCategory,
    pub cost: u32,
    pub infectivity: f32,
    pub severity: f32,
    pub lethality: f32,
    pub requires: Vec<&'static str>,
    pub description: &'static str,
}

pub fn all_upgrades() -> Vec<Upgrade> {
    vec![
        // === TRANSMISSION === (costs increased ~3x — DNA must be precious)
        Upgrade {
            id: "air1", name: "Air 1", category: UpgradeCategory::Transmission,
            cost: 18, infectivity: 3.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "Infectious particles spread through the air.",
        },
        Upgrade {
            id: "air2", name: "Air 2", category: UpgradeCategory::Transmission,
            cost: 28, infectivity: 7.0, severity: 0.0, lethality: 0.0,
            requires: vec!["air1"], description: "Enhanced airborne transmission.",
        },
        Upgrade {
            id: "water1", name: "Water 1", category: UpgradeCategory::Transmission,
            cost: 18, infectivity: 3.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "Pathogen survives in water. Ship transmission.",
        },
        Upgrade {
            id: "water2", name: "Water 2", category: UpgradeCategory::Transmission,
            cost: 28, infectivity: 7.0, severity: 0.0, lethality: 0.0,
            requires: vec!["water1"], description: "Enhanced waterborne transmission.",
        },
        Upgrade {
            id: "insect1", name: "Insect 1", category: UpgradeCategory::Transmission,
            cost: 20, infectivity: 5.0, severity: 1.0, lethality: 0.0,
            requires: vec![], description: "Insects carry the pathogen. Hot climate bonus.",
        },
        Upgrade {
            id: "insect2", name: "Insect 2", category: UpgradeCategory::Transmission,
            cost: 14, infectivity: 10.0, severity: 1.0, lethality: 0.0,
            requires: vec!["insect1"], description: "Enhanced insect vector.",
        },
        Upgrade {
            id: "bird1", name: "Bird 1", category: UpgradeCategory::Transmission,
            cost: 10, infectivity: 4.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "Birds spread pathogen between countries.",
        },
        Upgrade {
            id: "bird2", name: "Bird 2", category: UpgradeCategory::Transmission,
            cost: 15, infectivity: 7.0, severity: 0.0, lethality: 0.0,
            requires: vec!["bird1"], description: "Enhanced bird migration spread.",
        },
        Upgrade {
            id: "blood1", name: "Blood 1", category: UpgradeCategory::Transmission,
            cost: 10, infectivity: 5.0, severity: 2.0, lethality: 0.0,
            requires: vec![], description: "Bloodborne transmission.",
        },
        Upgrade {
            id: "blood2", name: "Blood 2", category: UpgradeCategory::Transmission,
            cost: 15, infectivity: 10.0, severity: 2.0, lethality: 0.0,
            requires: vec!["blood1"], description: "Enhanced bloodborne transmission.",
        },
        Upgrade {
            id: "rodent1", name: "Rodent 1", category: UpgradeCategory::Transmission,
            cost: 8, infectivity: 4.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "Rodents spread in urban areas.",
        },
        Upgrade {
            id: "rodent2", name: "Rodent 2", category: UpgradeCategory::Transmission,
            cost: 12, infectivity: 8.0, severity: 0.0, lethality: 0.0,
            requires: vec!["rodent1"], description: "Enhanced rodent transmission.",
        },

        // === SYMPTOMS ===
        // Mild
        Upgrade {
            id: "coughing", name: "Coughing", category: UpgradeCategory::Symptom,
            cost: 3, infectivity: 2.0, severity: 1.0, lethality: 0.0,
            requires: vec![], description: "Persistent cough. Airborne transmission.",
        },
        Upgrade {
            id: "nausea", name: "Nausea", category: UpgradeCategory::Symptom,
            cost: 3, infectivity: 1.0, severity: 1.0, lethality: 0.0,
            requires: vec![], description: "Feeling sick. Mild severity.",
        },
        Upgrade {
            id: "rash", name: "Rash", category: UpgradeCategory::Symptom,
            cost: 3, infectivity: 1.0, severity: 1.0, lethality: 0.0,
            requires: vec![], description: "Skin irritation. Visible symptom.",
        },
        Upgrade {
            id: "insomnia", name: "Insomnia", category: UpgradeCategory::Symptom,
            cost: 3, infectivity: 0.0, severity: 1.0, lethality: 0.0,
            requires: vec![], description: "Difficulty sleeping. Delays cure.",
        },
        Upgrade {
            id: "cysts", name: "Cysts", category: UpgradeCategory::Symptom,
            cost: 4, infectivity: 1.0, severity: 2.0, lethality: 0.0,
            requires: vec![], description: "Painful lumps. Severity increase.",
        },

        // Moderate
        Upgrade {
            id: "pneumonia", name: "Pneumonia", category: UpgradeCategory::Symptom,
            cost: 6, infectivity: 3.0, severity: 2.0, lethality: 1.0,
            requires: vec!["coughing"], description: "Lung infection. Cold climate bonus.",
        },
        Upgrade {
            id: "vomiting", name: "Vomiting", category: UpgradeCategory::Symptom,
            cost: 5, infectivity: 3.0, severity: 2.0, lethality: 0.0,
            requires: vec!["nausea"], description: "Severe nausea. Increases infectivity.",
        },
        Upgrade {
            id: "sweating", name: "Sweating", category: UpgradeCategory::Symptom,
            cost: 5, infectivity: 1.0, severity: 1.0, lethality: 0.0,
            requires: vec!["rash"], description: "Fever. Cold resistance.",
        },
        Upgrade {
            id: "paranoia", name: "Paranoia", category: UpgradeCategory::Symptom,
            cost: 6, infectivity: 0.0, severity: 2.0, lethality: 0.0,
            requires: vec!["insomnia"], description: "Delays cure research.",
        },
        Upgrade {
            id: "abscesses", name: "Abscesses", category: UpgradeCategory::Symptom,
            cost: 5, infectivity: 1.0, severity: 3.0, lethality: 0.0,
            requires: vec!["cysts"], description: "Infected wounds. High severity.",
        },

        // Severe
        Upgrade {
            id: "pulmonary_fibrosis", name: "Pulmonary Fibrosis", category: UpgradeCategory::Symptom,
            cost: 10, infectivity: 2.0, severity: 3.0, lethality: 3.0,
            requires: vec!["pneumonia"], description: "Lung scarring. High lethality.",
        },
        Upgrade {
            id: "diarrhea", name: "Diarrhea", category: UpgradeCategory::Symptom,
            cost: 7, infectivity: 5.0, severity: 2.0, lethality: 1.0,
            requires: vec!["vomiting"], description: "Dehydration. Very infectious.",
        },
        Upgrade {
            id: "skin_lesions", name: "Skin Lesions", category: UpgradeCategory::Symptom,
            cost: 8, infectivity: 4.0, severity: 4.0, lethality: 1.0,
            requires: vec!["sweating"], description: "Open sores. Highly visible.",
        },
        Upgrade {
            id: "seizures", name: "Seizures", category: UpgradeCategory::Symptom,
            cost: 8, infectivity: 1.0, severity: 4.0, lethality: 2.0,
            requires: vec!["paranoia"], description: "Neurological damage.",
        },
        Upgrade {
            id: "necrosis", name: "Necrosis", category: UpgradeCategory::Symptom,
            cost: 12, infectivity: 2.0, severity: 5.0, lethality: 5.0,
            requires: vec!["abscesses"], description: "Tissue death. Very lethal.",
        },

        // Lethal
        Upgrade {
            id: "total_organ_failure", name: "Total Organ Failure", category: UpgradeCategory::Symptom,
            cost: 18, infectivity: 0.0, severity: 8.0, lethality: 12.0,
            requires: vec!["pulmonary_fibrosis"], description: "Complete organ collapse. End-stage.",
        },
        Upgrade {
            id: "hemorrhagic_shock", name: "Hemorrhagic Shock", category: UpgradeCategory::Symptom,
            cost: 15, infectivity: 0.0, severity: 6.0, lethality: 10.0,
            requires: vec!["diarrhea"], description: "Bleeding out. Very lethal.",
        },
        Upgrade {
            id: "coma", name: "Coma", category: UpgradeCategory::Symptom,
            cost: 14, infectivity: 0.0, severity: 7.0, lethality: 8.0,
            requires: vec!["seizures"], description: "Unconsciousness. Slows cure.",
        },
        Upgrade {
            id: "immune_suppression", name: "Immune Suppression", category: UpgradeCategory::Symptom,
            cost: 12, infectivity: 3.0, severity: 5.0, lethality: 4.0,
            requires: vec!["necrosis"], description: "Destroys immune system.",
        },
        Upgrade {
            id: "dysentery", name: "Dysentery", category: UpgradeCategory::Symptom,
            cost: 10, infectivity: 4.0, severity: 4.0, lethality: 6.0,
            requires: vec!["diarrhea"], description: "Bloody diarrhea. Lethal dehydration.",
        },

        // === ABILITIES ===
        Upgrade {
            id: "drug_resistance1", name: "Drug Resistance 1", category: UpgradeCategory::Ability,
            cost: 12, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "+50% infection in wealthy countries.",
        },
        Upgrade {
            id: "drug_resistance2", name: "Drug Resistance 2", category: UpgradeCategory::Ability,
            cost: 18, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec!["drug_resistance1"], description: "+100% infection in wealthy countries.",
        },
        Upgrade {
            id: "cold_resistance1", name: "Cold Resistance 1", category: UpgradeCategory::Ability,
            cost: 10, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "+50% infection in cold climates.",
        },
        Upgrade {
            id: "cold_resistance2", name: "Cold Resistance 2", category: UpgradeCategory::Ability,
            cost: 15, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec!["cold_resistance1"], description: "+100% infection in cold climates.",
        },
        Upgrade {
            id: "heat_resistance1", name: "Heat Resistance 1", category: UpgradeCategory::Ability,
            cost: 10, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "+50% infection in hot climates.",
        },
        Upgrade {
            id: "heat_resistance2", name: "Heat Resistance 2", category: UpgradeCategory::Ability,
            cost: 15, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec!["heat_resistance1"], description: "+100% infection in hot climates.",
        },
        Upgrade {
            id: "genetic_hardening1", name: "Genetic Hardening 1", category: UpgradeCategory::Ability,
            cost: 15, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec![], description: "Cure research speed -5%.",
        },
        Upgrade {
            id: "genetic_hardening2", name: "Genetic Hardening 2", category: UpgradeCategory::Ability,
            cost: 20, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec!["genetic_hardening1"], description: "Cure research speed -10%.",
        },
        Upgrade {
            id: "genetic_reshuffle1", name: "Genetic Reshuffle", category: UpgradeCategory::Ability,
            cost: 30, infectivity: 0.0, severity: 0.0, lethality: 0.0,
            requires: vec!["genetic_hardening2"], description: "Resets cure progress by 25%.",
        },
    ]
}

#[derive(Debug, Clone)]
pub struct Disease {
    pub name: String,
    pub pathogen_type: PathogenType,
    pub upgrades: HashMap<String, bool>, // id -> unlocked
    pub total_infectivity: f32,
    pub total_severity: f32,
    pub total_lethality: f32,
    pub cure_slowdown: f32, // percentage reduction in cure speed
}

impl Disease {
    pub fn new(name: &str, pathogen_type: PathogenType) -> Self {
        Self {
            name: name.to_string(),
            pathogen_type,
            upgrades: HashMap::new(),
            total_infectivity: pathogen_type.base_infectivity(),
            total_severity: pathogen_type.base_severity(),
            total_lethality: pathogen_type.base_lethality(),
            cure_slowdown: 0.0,
        }
    }

    pub fn has_upgrade(&self, id: &str) -> bool {
        self.upgrades.get(id).copied().unwrap_or(false)
    }

    pub fn can_unlock(&self, upgrade: &Upgrade) -> bool {
        if self.has_upgrade(upgrade.id) {
            return false;
        }
        upgrade.requires.iter().all(|req| self.has_upgrade(req))
    }

    pub fn unlock(&mut self, upgrade: &Upgrade) {
        self.upgrades.insert(upgrade.id.to_string(), true);
        self.total_infectivity += upgrade.infectivity;
        self.total_severity += upgrade.severity;
        self.total_lethality += upgrade.lethality;

        // Genetic hardening reduces cure speed
        match upgrade.id {
            "genetic_hardening1" => self.cure_slowdown += 5.0,
            "genetic_hardening2" => self.cure_slowdown += 10.0,
            "genetic_reshuffle1" => self.cure_slowdown += 0.0, // special: resets cure
            _ => {}
        }
    }

    pub fn effective_infectivity(&self) -> f32 {
        self.total_infectivity.max(0.0)
    }

    pub fn effective_severity(&self) -> f32 {
        self.total_severity.max(0.0)
    }

    pub fn effective_lethality(&self) -> f32 {
        self.total_lethality.max(0.0)
    }
}
