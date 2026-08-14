# Epidemic NS — Game Design Document

## Overview

**Epidemic NS** (Natural Strategies) is an open-source pandemic simulation game inspired by Plague Inc. You play as a pathogen trying to infect and kill every human on Earth before scientists develop a cure.

**Platform:** Linux (Windows/Mac planned), Android (planned)
**Engine:** Rust + wgpu (custom)
**UI:** egui with Moo UI dark theme
**License:** MIT

---

## How to Play

### Starting a Game

1. **Title Screen** — Click "NEW GAME"
2. **Choose Pathogen** — Select from 7 disease types, each with unique mechanics
3. **Choose Difficulty** — Casual, Normal, Brutal, or Mega Brutal
4. **Choose Starting Country** — Click any country on the world map to place Patient Zero

### Gameplay Loop

1. Your disease starts in the chosen country with 1 infected person
2. Infection spreads within the country based on your disease's **Infectivity** stat
3. As infection grows, you earn **DNA Points** (currency for upgrades)
4. Spend DNA on **Transmission**, **Symptom**, and **Ability** upgrades
5. Infection spreads to neighboring countries
6. Countries close borders, research a cure
7. **Win** when every human is dead. **Lose** when the cure reaches 100%

---

## Controls

| Key | Action |
|-----|--------|
| **Mouse Click** | Select country (during origin selection) |
| **1** | Set speed to 1x |
| **2** | Set speed to 2x |
| **3** | Set speed to 3x |
| **Space** | Pause / Unpause |
| **Escape** | Quit |

---

## Pathogen Types

### Bacteria
- **Difficulty:** Beginner
- **Mechanic:** Standard pathogen. Cheap to devolve unwanted traits.
- **Strategy:** Balanced approach. Good for learning the game.
- **Base Stats:** Infectivity 1.0, Severity 1.0, Lethality 0.0

### Virus
- **Difficulty:** Intermediate
- **Mechanic:** Random mutations appear for free without spending DNA. Mutations are uncontrollable — you might get lethal symptoms before you're ready.
- **Strategy:** Evolve fast but carefully. Devolve unwanted mutations.
- **Base Stats:** Infectivity 1.1, Severity 1.2, Lethality 0.0

### Fungus
- **Difficulty:** Hard
- **Mechanic:** Very slow natural spread between countries. Can launch spores to specific countries.
- **Strategy:** Patient play. Infect quietly, then burst spread with spore abilities.
- **Base Stats:** Infectivity 0.5, Severity 0.8, Lethality 0.0

### Parasite
- **Difficulty:** Hard
- **Mechanic:** Naturally low severity — harder for the cure to start. Symbiosis suppresses symptoms.
- **Strategy:** Ultimate stealth. Infect everyone before revealing symptoms.
- **Base Stats:** Infectivity 0.8, Severity 0.3, Lethality 0.0

### Prion
- **Difficulty:** Hard
- **Mechanic:** Slow infection rate. Slows cure research once symptoms appear.
- **Strategy:** Long game. Build up slowly while sabotaging cure progress.
- **Base Stats:** Infectivity 0.6, Severity 0.7, Lethality 0.0

### Nano-Virus
- **Difficulty:** Expert
- **Mechanic:** Cure research starts immediately at game start. Must race against time.
- **Strategy:** Aggressive. Infect fast and kill fast before cure completes.
- **Base Stats:** Infectivity 1.2, Severity 1.5, Lethality 0.0

### Bio-Weapon
- **Difficulty:** Expert
- **Mechanic:** Innate lethality — kills automatically even without symptoms. Must suppress with gene compression.
- **Strategy:** Balance spreading and killing. Too lethal = kills before spreading globally.
- **Base Stats:** Infectivity 1.0, Severity 1.0, Lethality 0.1

---

## Upgrade Trees

### Transmission (How It Spreads)

Transmission upgrades increase your **Infectivity** stat, making the disease spread faster within and between countries.

| Upgrade | Cost | Infectivity | Severity | Prerequisite | Effect |
|---------|------|-------------|----------|--------------|--------|
| Air 1 | 9 | +3.0 | — | — | Airborne particles |
| Air 2 | 13 | +7.0 | — | Air 1 | Enhanced airborne |
| Water 1 | 9 | +3.0 | — | — | Waterborne, ship transmission |
| Water 2 | 13 | +7.0 | — | Water 1 | Enhanced waterborne |
| Insect 1 | 9 | +5.0 | +1.0 | — | Insect vector, hot climate bonus |
| Insect 2 | 14 | +10.0 | +1.0 | Insect 1 | Enhanced insect |
| Bird 1 | 10 | +4.0 | — | — | Bird migration between countries |
| Bird 2 | 15 | +7.0 | — | Bird 1 | Enhanced bird spread |
| Blood 1 | 10 | +5.0 | +2.0 | — | Bloodborne transmission |
| Blood 2 | 15 | +10.0 | +2.0 | Blood 1 | Enhanced bloodborne |
| Rodent 1 | 8 | +4.0 | — | — | Urban rodent spread |
| Rodent 2 | 12 | +8.0 | — | Rodent 1 | Enhanced rodent |

### Symptoms (What It Does)

Symptoms increase **Severity** (triggers cure research) and **Lethality** (kills hosts). Organized in 4 tiers from mild to lethal.

**Tier 1 — Mild (Low cost, low severity):**

| Upgrade | Cost | Infectivity | Severity | Lethality | Prerequisite |
|---------|------|-------------|----------|-----------|--------------|
| Coughing | 3 | +2.0 | +1.0 | — | — |
| Nausea | 3 | +1.0 | +1.0 | — | — |
| Rash | 3 | +1.0 | +1.0 | — | — |
| Insomnia | 3 | — | +1.0 | — | — |
| Cysts | 4 | +1.0 | +2.0 | — | — |

**Tier 2 — Moderate:**

| Upgrade | Cost | Infectivity | Severity | Lethality | Prerequisite |
|---------|------|-------------|----------|-----------|--------------|
| Pneumonia | 6 | +3.0 | +2.0 | +1.0 | Coughing |
| Vomiting | 5 | +3.0 | +2.0 | — | Nausea |
| Sweating | 5 | +1.0 | +1.0 | — | Rash |
| Paranoia | 6 | — | +2.0 | — | Insomnia |
| Abscesses | 5 | +1.0 | +3.0 | — | Cysts |

**Tier 3 — Severe:**

| Upgrade | Cost | Infectivity | Severity | Lethality | Prerequisite |
|---------|------|-------------|----------|-----------|--------------|
| Pulmonary Fibrosis | 10 | +2.0 | +3.0 | +3.0 | Pneumonia |
| Diarrhea | 7 | +5.0 | +2.0 | +1.0 | Vomiting |
| Skin Lesions | 8 | +4.0 | +4.0 | +1.0 | Sweating |
| Seizures | 8 | +1.0 | +4.0 | +2.0 | Paranoia |
| Necrosis | 12 | +2.0 | +5.0 | +5.0 | Abscesses |

**Tier 4 — Lethal:**

| Upgrade | Cost | Infectivity | Severity | Lethality | Prerequisite |
|---------|------|-------------|----------|-----------|--------------|
| Total Organ Failure | 18 | — | +8.0 | +12.0 | Pulmonary Fibrosis |
| Hemorrhagic Shock | 15 | — | +6.0 | +10.0 | Diarrhea |
| Coma | 14 | — | +7.0 | +8.0 | Seizures |
| Immune Suppression | 12 | +3.0 | +5.0 | +4.0 | Necrosis |
| Dysentery | 10 | +4.0 | +4.0 | +6.0 | Diarrhea |

### Abilities (Passive Buffs)

Abilities provide passive bonuses. They don't increase infectivity/severity/lethality directly.

| Upgrade | Cost | Effect | Prerequisite |
|---------|------|--------|--------------|
| Drug Resistance 1 | 12 | +50% infection rate in wealthy countries | — |
| Drug Resistance 2 | 18 | +100% infection rate in wealthy countries | Drug Resistance 1 |
| Cold Resistance 1 | 10 | +50% infection rate in cold climates | — |
| Cold Resistance 2 | 15 | +100% infection rate in cold climates | Cold Resistance 1 |
| Heat Resistance 1 | 10 | +50% infection rate in hot climates | — |
| Heat Resistance 2 | 15 | +100% infection rate in hot climates | Heat Resistance 1 |
| Genetic Hardening 1 | 15 | Cure research speed -5% | — |
| Genetic Hardening 2 | 20 | Cure research speed -10% | Genetic Hardening 1 |
| Genetic Reshuffle | 30 | Resets cure progress by 25% | Genetic Hardening 2 |

---

## World Map

The map shows 64 regions covering the entire world. Each region has:
- **Population** — Total people (2026 estimates)
- **Infected** — Currently infected count
- **Dead** — Death count
- **Borders** — Open or closed
- **Cure Contribution** — Research progress

### Region List

**North America:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| United States | US | 341.8M | Wealthy, high drug resistance |
| Canada | CA | 41M | Wealthy, cold climate |
| Mexico | MX | 130M | Hot climate |
| Central America | CAM | 55M | Includes Caribbean islands |

**South America:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| Brazil | BR | 216M | Hot climate, large population |
| Argentina | AR | 47M | |
| Colombia | CO | 52M | Hot climate |
| Peru | PE | 34M | Hot climate |
| Venezuela | VE | 28M | |
| South America Rest | SA | 45M | Chile, Ecuador, Bolivia, etc. |

**Europe:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| United Kingdom | GB | 69M | Wealthy, island (harder to infect) |
| France | FR | 68M | Wealthy |
| Germany | DE | 84M | Wealthy, central hub |
| Spain | ES | 48M | |
| Portugal | PT | 10.4M | |
| Italy | IT | 59M | Wealthy |
| Western Europe | WE | 75M | Netherlands, Belgium, Switzerland, Austria, Ireland |
| Northern Europe | NE | 28M | Sweden, Norway, Denmark, Finland, Iceland |
| Poland | PL | 38M | |
| Ukraine | UA | 37M | |
| Eastern Europe | EE | 85M | 21 countries grouped |

**Russia & Central Asia:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| Russia | RU | 144M | Huge landmass, cold |
| Kazakhstan | KZ | 20M | |
| Central Asia Rest | CA2 | 60M | Uzbekistan, Turkmenistan, etc. |

**Middle East:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| Turkey | TR | 86M | |
| Saudi Arabia | SA2 | 37M | Air travel hub |
| Iran | IR | 88M | |
| Iraq | IQ | 43M | |
| Middle East Rest | ME | 65M | UAE, Israel, Jordan, etc. |

**Africa:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| Egypt | EG | 106M | Hot climate |
| Algeria | DZ | 46M | Hot climate |
| Morocco | MA | 37.5M | |
| North Africa Rest | NA | 45M | Tunisia, Libya, Sudan, South Sudan |
| Nigeria | NG | 224M | Hot climate, huge population |
| Ghana | GH | 34M | |
| West Africa Rest | WA | 180M | 15 countries grouped |
| Ethiopia | ET | 126M | Hot climate |
| Kenya | KE | 56M | |
| Tanzania | TZ | 65M | |
| East Africa Rest | EA | 140M | 11 countries grouped |
| DR Congo | CD | 102M | Hot climate |
| Central Africa Rest | CF | 70M | 8 countries grouped |
| South Africa | ZA | 62M | |
| Southern Africa Rest | SA3 | 30M | |

**South Asia:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| India | IN | 1,450M | Massive population, hot |
| Pakistan | PK | 240M | |
| Bangladesh | BD | 175M | |
| South Asia Rest | SA4 | 55M | Nepal, Sri Lanka, Bhutan, Maldives |

**Southeast Asia:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| Indonesia | ID | 280M | Island nation, hot |
| Thailand | TH | 72M | |
| Vietnam | VN | 100M | |
| Philippines | PH | 117M | Island nation |
| Myanmar | MM | 55M | |
| Malaysia | MY | 34M | |
| Southeast Asia Rest | SEA | 60M | Cambodia, Laos, Brunei, etc. |

**East Asia:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| China | CN | 1,425M | Massive population |
| Japan | JP | 124M | Wealthy, island |
| South Korea | KR | 52M | Wealthy |
| North Korea | KP | 26M | Isolated |
| Taiwan | TW | 24M | |

**Oceania:**
| Region | Code | Population | Notes |
|--------|------|------------|-------|
| Australia | AU | 27M | Wealthy, hot |
| New Zealand | NZ | 5.2M | Island |
| Oceania Rest | OC | 15M | Pacific islands |
| Greenland | GL | 57K | Very isolated, cold |

---

## Simulation Mechanics

### Infection Spread

Each tick (60ms at 1x speed), every infected region calculates:

```
new_infections = infected × 0.0008 × infectivity × drug_modifier × (healthy ÷ population)
```

**Drug Modifier:**
- Wealthy countries (US, GB, DE, FR, JP, KR, AU, CA, IT, ES, NL, SE, CH, WE, NE): base 0.5x resistance
- With Drug Resistance 1: 1.5x (counters the resistance)
- With Drug Resistance 2: 2.0x (overcomes resistance entirely)

**Key insight:** Infectivity upgrades make the disease spread faster. The `healthy/population` factor means spread slows as more people are already infected.

### Death Rate

```
new_deaths = infected × 0.00002 × lethality
```

Lethality comes from:
- Bio-Weapon base: 0.1
- Symptom upgrades (Necrosis: +5.0, Total Organ Failure: +12.0, etc.)

**Key insight:** Lethality kills infected people. If you kill too fast, you run out of hosts before spreading globally.

### Cross-Border Spread

When a region has >1% infection:
```
chance = infection_percentage × 0.001
```

If the random check passes and borders are open, the neighbor gets seeded with 1 infected person.

**Key insight:** Bird transmission upgrades increase cross-border spread. Countries closing borders blocks this entirely.

### Cure Research

Starts when:
- 3+ countries are infected
- Global severity > 0.5

Speed per region:
- Wealthy countries: 0.008/tick
- Others: 0.003/tick
- Scaled by severity factor (severity ÷ 10, max 2.0)
- Reduced by dead researchers (dead% × 0.5, min 0.3)
- Reduced by Genetic Hardening upgrades

**Cure milestones:** 10%, 25%, 50%, 75%, 90% — each triggers a news headline.

**Cure at 100% = you lose.**

### Border Closure

Triggered when severity ≥ 1.0 and an infected neighbor exists.

```
close_chance = severity ÷ 50 (max 0.1)
```

Modified by difficulty:
- Casual: 0.5x
- Normal: 1.0x
- Brutal: 1.5x
- Mega Brutal: 2.0x

Closed borders block cross-border infection spread.

### DNA Points

**Passive income:** Every 50 ticks:
```
dna_earned = 1 + min(total_infected ÷ 10,000,000, 5)
```

**Bubble income:** DNA bubbles appear on the map (every 30 ticks, max 5). Click to collect. Each bubble is worth:
```
bubble_value = 1 + severity ÷ 3
```

Bubbles expire after 200 ticks.

### Random Events

Every 100 ticks:
- **15% chance:** Sports event — infection spikes in a random infected country (+0.1% of population)
- **10% chance:** Research funding — cure progress +2.0%

---

## Difficulty Levels

| Difficulty | Cure Speed | Border Close Rate | Description |
|------------|------------|-------------------|-------------|
| Casual | 0.5x | 0.5x | Slower cure, weaker borders |
| Normal | 1.0x | 1.0x | Standard challenge |
| Brutal | 1.5x | 1.5x | Faster cure, stronger borders |
| Mega Brutal | 2.0x | 2.0x | Extreme difficulty |

---

## Win/Lose Conditions

### Win
Every human on Earth is dead. All 64 regions have `healthy = 0`.

### Lose
Cure progress reaches 100%.

### Edge Cases
- If your pathogen kills all infected before spreading to new countries, the disease dies out (currently not tracked as a loss condition).
- If you close all borders by killing researchers, cure slows but doesn't stop.

---

## Strategy Guide

### Beginner Strategy (Bacteria, Normal)
1. Start in **Saudi Arabia** (air travel hub, connects to many countries)
2. Evolve **Air 1** and **Water 1** for transmission
3. Evolve mild symptoms (**Coughing**, **Nausea**) for infectivity
4. Wait until 20+ countries infected
5. Evolve **Drug Resistance 1** to overcome wealthy country resistance
6. Once globally spread, evolve lethal symptoms (**Total Organ Failure**)
7. Evolve **Genetic Hardening** to slow cure

### Stealth Strategy (Parasite, Brutal)
1. Start in **India** or **China** (huge populations)
2. Evolve only transmission, no symptoms
3. **Symbiosis** keeps severity low — cure doesn't start
4. Infect every country silently
5. Once globally spread, break symbiosis and evolve lethal symptoms all at once
6. Kill everyone before cure can react

### Speed Strategy (Nano-Virus, Normal)
1. Start in **Egypt** (central location)
2. Cure starts immediately — you're on a timer
3. Evolve **Air 1**, **Air 2**, **Blood 1** fast
4. Keep severity low to slow cure
5. Race to infect globally before cure completes
6. Add lethality late

---

## HUD Reference

### Left Panel
- **EPIDEMIC NS** — Game title
- **Tick** — Current simulation tick
- **Speed** — Current game speed (1x/2x/3x)
- **Population Card:**
  - Healthy (green) — Uninfected people
  - Infected (red) — Currently sick
  - Dead (gray) — Deceased
- **DNA Points** — Currency for upgrades
- **Cure Progress** — Bar + percentage (lose at 100%)
- **Disease Card:**
  - Infectivity — How fast it spreads
  - Severity — How visible/dangerous (triggers cure)
  - Lethality — How deadly
- **Speed Buttons** — 1x, 2x, 3x
- **Phase Indicator** — Current game state

### Right Panel (Evolution Menu)
- **DNA** — Current points
- **Transmission** — Green section, 12 upgrades
- **Symptoms** — Amber section, 19 upgrades
- **Abilities** — Blue section, 9 upgrades
- ✓ = owned, clickable = affordable, grayed = locked/unaffordable

### Bottom Bar
- News ticker with latest events

### Map
- **Dark green** — Healthy region
- **Red gradient** — Infected (darker red = more infected)
- **Dark gray** — Fallen (everyone dead)
- **Dark outlines** — Country borders
- **White tint** — Hovered country

---

## Version History

### v0.1.0 (Current)
- Initial release
- 7 pathogen types
- 35 upgrades (transmission/symptoms/abilities)
- 64 regions with 2026 population data
- SVG world map with real country shapes
- Disease evolution system
- Cure research + border closures
- Events system (sports, research boosts)
- egui HUD with Moo UI dark theme
- Title screen, pathogen select, difficulty select
- Speed controls (1x/2x/3x)
- Country border outlines + hover tinting

### Planned Features
- Country detail panel with infection history graph
- Transmission animations (planes/ships flying between countries)
- DNA bubble click-to-collect on map
- Map zoom/pan
- Country hover tooltip with stats
- Score system with biohazard ratings
- News ticker scrolling
- Better font / UI polish
- Audio
- Android touch controls
- Save/load system
- Multiplayer
