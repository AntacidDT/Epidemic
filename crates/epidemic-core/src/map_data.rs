/// Simplified world map data — continent grid + country definitions.
/// Grid is 80x45, each cell is either ocean or a country ID.

pub const MAP_W: usize = 80;
pub const MAP_H: usize = 45;

pub struct CountryDef {
    pub id: u8,
    pub name: &'static str,
    pub population: u32, // in thousands
    pub x: u8,           // approximate center for display
    pub y: u8,
}

/// All countries. Index = country ID (0 = ocean, 1..N = countries).
pub const COUNTRIES: &[CountryDef] = &[
    CountryDef { id: 0, name: "Ocean", population: 0, x: 0, y: 0 },
    // North America
    CountryDef { id: 1, name: "USA", population: 331000, x: 14, y: 14 },
    CountryDef { id: 2, name: "Canada", population: 38000, x: 14, y: 8 },
    CountryDef { id: 3, name: "Mexico", population: 128000, x: 12, y: 18 },
    // South America
    CountryDef { id: 4, name: "Brazil", population: 214000, x: 24, y: 30 },
    CountryDef { id: 5, name: "Argentina", population: 45000, x: 22, y: 37 },
    CountryDef { id: 6, name: "Colombia", population: 51000, x: 20, y: 24 },
    CountryDef { id: 7, name: "Peru", population: 33000, x: 20, y: 30 },
    // Europe
    CountryDef { id: 8, name: "UK", population: 67000, x: 38, y: 10 },
    CountryDef { id: 9, name: "France", population: 67000, x: 40, y: 13 },
    CountryDef { id: 10, name: "Germany", population: 83000, x: 43, y: 11 },
    CountryDef { id: 11, name: "Spain", population: 47000, x: 38, y: 15 },
    CountryDef { id: 12, name: "Italy", population: 60000, x: 44, y: 15 },
    CountryDef { id: 13, name: "Poland", population: 38000, x: 46, y: 10 },
    CountryDef { id: 14, name: "Sweden", population: 10000, x: 44, y: 6 },
    // Russia / Central Asia
    CountryDef { id: 15, name: "Russia", population: 144000, x: 58, y: 8 },
    // Middle East
    CountryDef { id: 16, name: "Turkey", population: 84000, x: 50, y: 15 },
    CountryDef { id: 17, name: "Saudi Arabia", population: 35000, x: 52, y: 19 },
    // Africa
    CountryDef { id: 18, name: "Egypt", population: 102000, x: 48, y: 20 },
    CountryDef { id: 19, name: "Nigeria", population: 211000, x: 42, y: 25 },
    CountryDef { id: 20, name: "South Africa", population: 60000, x: 46, y: 36 },
    CountryDef { id: 21, name: "DR Congo", population: 92000, x: 46, y: 28 },
    // South / East Asia
    CountryDef { id: 22, name: "India", population: 1393000, x: 60, y: 20 },
    CountryDef { id: 23, name: "China", population: 1412000, x: 68, y: 15 },
    CountryDef { id: 24, name: "Japan", population: 125000, x: 74, y: 14 },
    CountryDef { id: 25, name: "Indonesia", population: 273000, x: 70, y: 28 },
    CountryDef { id: 26, name: "Thailand", population: 70000, x: 67, y: 23 },
    // Oceania
    CountryDef { id: 27, name: "Australia", population: 26000, x: 74, y: 36 },
];

/// Get the map grid. Returns [MAP_W * MAP_H] country IDs.
/// 0 = ocean, 1..27 = country.
pub fn map_grid() -> Vec<u8> {
    let mut grid = vec![0u8; MAP_W * MAP_H];

    // Helper to fill a rect with a country id
    let mut fill = |id: u8, x1: usize, y1: usize, x2: usize, y2: usize| {
        for y in y1..=y2 {
            for x in x1..=x2 {
                if x < MAP_W && y < MAP_H {
                    grid[y * MAP_W + x] = id;
                }
            }
        }
    };

    // Simplified continent shapes
    // North America
    fill(2, 10, 5, 20, 10);   // Canada
    fill(1, 10, 11, 20, 17);  // USA
    fill(3, 11, 18, 16, 21);  // Mexico

    // Central America / Caribbean (part of Mexico for simplicity)
    fill(3, 16, 19, 19, 22);

    // South America
    fill(6, 18, 22, 23, 26);  // Colombia
    fill(7, 18, 27, 23, 32);  // Peru
    fill(4, 22, 27, 28, 34);  // Brazil
    fill(5, 20, 35, 25, 40);  // Argentina

    // Europe
    fill(8, 36, 8, 39, 11);   // UK
    fill(14, 42, 4, 46, 8);   // Sweden
    fill(10, 41, 9, 46, 13);  // Germany
    fill(9, 38, 12, 42, 16);  // France
    fill(11, 36, 14, 40, 18); // Spain
    fill(12, 42, 14, 46, 18); // Italy
    fill(13, 46, 9, 50, 13);  // Poland

    // Russia (big stretch across top)
    fill(15, 50, 4, 72, 12);

    // Middle East
    fill(16, 48, 14, 53, 18); // Turkey
    fill(17, 50, 18, 56, 22); // Saudi Arabia

    // Africa
    fill(18, 46, 19, 51, 23); // Egypt
    fill(19, 39, 23, 44, 28); // Nigeria
    fill(21, 44, 26, 49, 31); // DR Congo
    fill(20, 43, 33, 48, 38); // South Africa

    // South / East Asia
    fill(22, 57, 17, 65, 24); // India
    fill(23, 65, 11, 75, 20); // China
    fill(26, 65, 21, 70, 25); // Thailand
    fill(24, 73, 11, 77, 17); // Japan
    fill(25, 67, 26, 76, 31); // Indonesia

    // Oceania
    fill(27, 71, 33, 78, 39); // Australia

    grid
}

/// Neighbors list — which country IDs border each other.
/// Used for land-based transmission.
pub fn country_neighbors() -> Vec<Vec<u8>> {
    // Index = country id, value = list of neighbor country ids
    vec![
        vec![],                     // 0: ocean
        vec![2, 3],                 // 1: USA
        vec![1],                    // 2: Canada
        vec![1, 6],                 // 3: Mexico
        vec![5, 6, 7],              // 4: Brazil
        vec![4, 7],                 // 5: Argentina
        vec![3, 4, 7],              // 6: Colombia
        vec![4, 5, 6],              // 7: Peru
        vec![9],                    // 8: UK
        vec![8, 10, 11],            // 9: France
        vec![9, 12, 13, 14, 15],    // 10: Germany
        vec![9],                    // 11: Spain
        vec![10, 16],               // 12: Italy
        vec![10, 15],               // 13: Poland
        vec![10, 15],               // 14: Sweden
        vec![10, 13, 14, 16, 23],   // 15: Russia
        vec![12, 15, 17, 18],       // 16: Turkey
        vec![16, 18, 22],           // 17: Saudi Arabia
        vec![16, 17, 19, 21],       // 18: Egypt
        vec![18, 21],               // 19: Nigeria
        vec![21],                   // 20: South Africa
        vec![18, 19, 20],           // 21: DR Congo
        vec![17, 23, 26],           // 22: India
        vec![15, 22, 24, 25, 26],   // 23: China
        vec![23],                   // 24: Japan
        vec![23, 26, 27],           // 25: Indonesia
        vec![22, 23, 25],           // 26: Thailand
        vec![25],                   // 27: Australia
    ]
}
