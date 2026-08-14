use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Climate { Arid, Tropical, Temperate, Arctic }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Density { Megacity, Urban, Rural }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GovernmentType { Authoritarian, Democratic, Failed }

/// A game region — the core simulation unit.
#[derive(Debug, Clone)]
pub struct Region {
    // Identity
    pub id: u16,
    pub code: String,
    pub name: String,
    pub svg_codes: Vec<String>,

    // Population
    pub population: u64,
    pub infected: u64,
    pub dead: u64,

    // Geography
    pub climate: Climate,
    pub density: Density,
    pub is_island: bool,
    pub is_wealthy: bool,

    // Infrastructure
    pub hospital_capacity: f32,
    pub healthcare_collapse: bool,
    pub has_airport: bool,
    pub has_seaport: bool,
    pub borders_open: bool,
    pub air_borders_open: bool,
    pub sea_borders_open: bool,

    // Society
    pub panic: f32,
    pub government_type: GovernmentType,
    pub misinformation: f32,
    pub lockdown_level: f32,

    // Cure
    pub cure_progress: f32,
    pub vaccine_doses: u64,
    pub vaccinated: u64,

    // State
    pub fallen: bool,

    // Supply chain
    pub manufacturing_capacity: f32,
    pub agricultural_capacity: f32,

    // History
    pub infection_history: Vec<(u64, u64)>,
    pub death_history: Vec<(u64, u64)>,
}

impl Region {
    pub fn new(id: u16, code: &str, name: &str, population: u64, svg_codes: &[&str]) -> Self {
        Self {
            id, code: code.to_string(), name: name.to_string(), population,
            infected: 0, dead: 0,
            climate: Climate::Temperate, density: Density::Urban,
            is_island: false, is_wealthy: false,
            hospital_capacity: 3.0, healthcare_collapse: false,
            has_airport: false, has_seaport: false,
            borders_open: true, air_borders_open: true, sea_borders_open: true,
            panic: 0.0, government_type: GovernmentType::Democratic,
            misinformation: 0.0, lockdown_level: 0.0,
            cure_progress: 0.0, vaccine_doses: 0, vaccinated: 0,
            fallen: false,
            manufacturing_capacity: 0.3, agricultural_capacity: 0.3,
            infection_history: Vec::new(), death_history: Vec::new(),
            svg_codes: svg_codes.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn healthy(&self) -> u64 {
        self.population.saturating_sub(self.infected + self.dead)
    }

    pub fn infection_pct(&self) -> f32 {
        if self.population == 0 { return 0.0; }
        self.infected as f32 / self.population as f32
    }

    pub fn death_pct(&self) -> f32 {
        if self.population == 0 { return 0.0; }
        self.dead as f32 / self.population as f32
    }

    pub fn is_overwhelmed(&self) -> bool {
        let beds = self.population as f32 * self.hospital_capacity / 1000.0;
        self.infected as f32 > beds
    }

    pub fn mortality_multiplier(&self) -> f32 {
        if self.healthcare_collapse { 3.0 }
        else if self.is_overwhelmed() { 1.5 }
        else { 1.0 }
    }

    pub fn record_history(&mut self, tick: u64) {
        if self.infection_history.len() >= 200 { self.infection_history.remove(0); }
        if self.death_history.len() >= 200 { self.death_history.remove(0); }
        self.infection_history.push((tick, self.infected));
        self.death_history.push((tick, self.dead));
    }
}

// Builder
struct B(Region);
fn r(id: u16, code: &str, name: &str, pop: u64, codes: &[&str]) -> B {
    B(Region::new(id, code, name, pop, codes))
}
impl B {
    fn w(mut self) -> Self { self.0.is_wealthy = true; self }
    fn i(mut self) -> Self { self.0.is_island = true; self }
    fn ar(mut self) -> Self { self.0.climate = Climate::Arid; self }
    fn tr(mut self) -> Self { self.0.climate = Climate::Tropical; self }
    fn te(mut self) -> Self { self.0.climate = Climate::Temperate; self }
    fn ac(mut self) -> Self { self.0.climate = Climate::Arctic; self }
    fn mc(mut self) -> Self { self.0.density = Density::Megacity; self }
    fn ur(mut self) -> Self { self.0.density = Density::Urban; self }
    fn ru(mut self) -> Self { self.0.density = Density::Rural; self }
    fn ap(mut self) -> Self { self.0.has_airport = true; self }
    fn sp(mut self) -> Self { self.0.has_seaport = true; self }
    fn au(mut self) -> Self { self.0.government_type = GovernmentType::Authoritarian; self }
    fn fs(mut self) -> Self { self.0.government_type = GovernmentType::Failed; self }
    fn h(mut self, c: f32) -> Self { self.0.hospital_capacity = c; self }
    fn m(mut self, c: f32) -> Self { self.0.manufacturing_capacity = c; self }
    fn ag(mut self, c: f32) -> Self { self.0.agricultural_capacity = c; self }
}
impl From<B> for Region { fn from(b: B) -> Self { b.0 } }

pub fn build_regions() -> Vec<Region> {
    vec![
        // ── North America ──
        r(1,"US","United States",341_800_000,&["US"]).w().te().mc().ap().sp().h(5.8).into(),
        r(2,"CA","Canada",41_000_000,&["CA"]).w().ac().ur().ap().sp().h(4.2).into(),
        r(3,"MX","Mexico",130_000_000,&["MX"]).tr().mc().ap().sp().h(2.1).into(),
        r(4,"CAM","Central America",35_000_000,&["GT","BZ","HN","SV","NI","CR","PA"]).tr().ur().ap().sp().h(1.5).into(),
        r(5,"CRB","Caribbean",20_000_000,&["CU","JM","HT","DO","TT","BB","GD","LC","VC","AG","KN","DM","BS","PR","VI","AI","AW","CW","SX","MF","BL","PM"]).tr().ur().ap().sp().i().h(1.2).into(),

        // ── South America ──
        r(6,"BR","Brazil",216_000_000,&["BR"]).tr().mc().ap().sp().h(2.3).m(0.7).ag(0.8).into(),
        r(7,"AR","Argentina",47_000_000,&["AR"]).te().ur().ap().sp().h(3.0).into(),
        r(8,"CO","Colombia",52_000_000,&["CO"]).tr().ur().ap().sp().h(1.8).into(),
        r(9,"PE","Peru",34_000_000,&["PE"]).tr().ur().ap().sp().h(1.6).into(),
        r(10,"VE","Venezuela",28_000_000,&["VE"]).tr().ur().ap().sp().h(1.0).fs().into(),
        r(11,"CL","Chile",19_500_000,&["CL"]).te().ur().ap().sp().h(3.0).into(),
        r(12,"BO","Bolivia",12_400_000,&["BO"]).tr().ru().h(1.2).into(),
        r(13,"PY","Paraguay",7_500_000,&["PY"]).te().ur().h(1.5).into(),
        r(14,"UY","Uruguay",3_400_000,&["UY"]).te().ur().ap().sp().h(2.5).into(),
        r(15,"EC","Ecuador",18_000_000,&["EC"]).tr().ur().ap().sp().h(1.4).into(),
        r(16,"GYA","Guyana-Suriname",1_500_000,&["GY","SR","GF"]).tr().ru().ap().h(0.8).into(),

        // ── British Isles ──
        r(17,"GB","United Kingdom",69_000_000,&["GB"]).w().te().mc().ap().sp().i().h(5.5).m(0.6).into(),
        r(18,"IE","Ireland",5_300_000,&["IE"]).w().te().ur().ap().sp().i().h(5.0).into(),

        // ── Western Europe ──
        r(19,"FR","France",68_000_000,&["FR"]).w().te().mc().ap().sp().h(6.0).m(0.6).into(),
        r(20,"DE","Germany",84_000_000,&["DE"]).w().te().mc().ap().sp().h(8.0).m(0.8).into(),
        r(21,"ES","Spain",48_000_000,&["ES"]).w().te().ur().ap().sp().h(4.5).into(),
        r(22,"PT","Portugal",10_400_000,&["PT"]).te().ur().ap().sp().h(3.5).into(),
        r(23,"IT","Italy",59_000_000,&["IT"]).w().te().mc().ap().sp().h(5.0).into(),
        r(24,"BNL","Benelux",30_000_000,&["NL","BE","LU"]).w().te().ur().ap().sp().h(6.5).into(),
        r(25,"CH","Switzerland",9_000_000,&["CH"]).w().te().ur().ap().h(8.0).into(),

        // ── Scandinavia ──
        r(26,"SCA","Scandinavia",28_000_000,&["SE","NO","DK","FI","IS"]).w().ac().ur().ap().sp().i().h(5.0).into(),

        // ── Central Europe ──
        r(27,"AT","Austria",9_200_000,&["AT"]).w().te().ur().ap().h(7.0).into(),
        r(28,"PL","Poland",38_000_000,&["PL"]).te().ur().ap().h(3.5).into(),
        r(29,"CZ","Czech Republic",10_900_000,&["CZ"]).te().ur().ap().h(4.0).into(),
        r(30,"SK","Slovakia",5_500_000,&["SK"]).te().ur().ap().h(3.5).into(),
        r(31,"HU","Hungary",9_700_000,&["HU"]).te().ur().ap().h(3.5).into(),

        // ── Balkans ──
        r(32,"HR","Croatia",3_900_000,&["HR"]).te().ur().ap().h(3.5).into(),
        r(33,"RS","Serbia",6_600_000,&["RS"]).te().ur().ap().h(3.0).into(),
        r(34,"RO","Romania",19_000_000,&["RO"]).te().ur().ap().h(3.0).into(),
        r(35,"BG","Bulgaria",6_500_000,&["BG"]).te().ur().ap().h(3.0).into(),
        r(36,"BA","Bosnia",3_200_000,&["BA"]).te().ur().ap().h(2.5).into(),
        r(37,"SI","Slovenia",2_100_000,&["SI"]).w().te().ur().ap().h(4.5).into(),
        r(38,"ALB","Albania-Kosovo",3_500_000,&["AL","XK"]).te().ur().ap().h(2.0).into(),
        r(39,"MK","North Macedonia",1_800_000,&["MK"]).te().ur().ap().h(2.5).into(),
        r(40,"ME","Montenegro",620_000,&["ME"]).te().ur().h(2.5).into(),

        // ── Eastern Europe ──
        r(41,"UA","Ukraine",37_000_000,&["UA"]).te().mc().ap().h(2.5).au().into(),
        r(42,"BY","Belarus",9_200_000,&["BY"]).te().ur().ap().h(2.5).au().into(),
        r(43,"MD","Moldova",2_600_000,&["MD"]).te().ur().h(2.0).into(),
        r(44,"BAL","Baltic States",6_100_000,&["EE","LV","LT"]).te().ur().ap().h(4.0).into(),

        // ── Russia ──
        r(45,"RU","Russia",144_000_000,&["RU"]).ac().mc().ap().sp().h(4.0).au().m(0.5).into(),

        // ── Caucasus ──
        r(46,"CAU","Caucasus",17_000_000,&["GE","AM","AZ"]).te().ur().ap().h(2.0).into(),

        // ── Middle East ──
        r(47,"TR","Turkey",86_000_000,&["TR"]).ar().mc().ap().sp().h(2.8).into(),
        r(48,"IR","Iran",88_000_000,&["IR"]).ar().mc().ap().sp().h(1.6).au().into(),
        r(49,"IQ","Iraq",43_000_000,&["IQ"]).ar().ur().ap().h(1.0).fs().into(),
        r(50,"SY","Syria",22_000_000,&["SY"]).ar().ur().ap().h(0.8).fs().into(),
        r(51,"LEV","Levant",18_000_000,&["IL","JO","LB","PS"]).ar().ur().ap().sp().h(2.5).into(),
        r(52,"YEM","Yemen",34_000_000,&["YE"]).ar().ru().ap().h(0.5).fs().into(),
        r(53,"GULF","Gulf States",18_000_000,&["AE","QA","BH","KW","OM"]).ar().ur().ap().sp().w().h(3.0).into(),
        r(54,"SA","Saudi Arabia",37_000_000,&["SA"]).ar().ur().ap().sp().w().h(2.5).into(),

        // ── North Africa ──
        r(55,"EG","Egypt",106_000_000,&["EG"]).ar().mc().ap().sp().h(1.6).au().into(),
        r(56,"LY","Libya",7_000_000,&["LY"]).ar().ur().ap().h(0.8).au().into(),
        r(57,"DZ","Algeria",46_000_000,&["DZ"]).ar().ur().ap().sp().h(1.5).au().into(),
        r(58,"MA","Morocco-Tunisia",53_000_000,&["MA","TN"]).ar().ur().ap().sp().h(1.2).into(),
        r(59,"WSH","Western Sahara",600_000,&["EH"]).ar().ru().h(0.3).into(),

        // ── West Africa ──
        r(60,"NG","Nigeria",224_000_000,&["NG"]).tr().mc().ap().sp().h(0.5).into(),
        r(61,"GH","Ghana",34_000_000,&["GH"]).tr().ur().ap().h(0.8).into(),
        r(62,"SEN","Senegal-Gambia",18_000_000,&["SN","GM","GW","CV"]).tr().ur().ap().sp().h(0.5).into(),
        r(63,"MLI","Mali-Niger",42_000_000,&["ML","NE","MR"]).tr().ru().ap().h(0.2).into(),
        r(64,"GUI","Guinea-Sierra Leone-Liberia",22_000_000,&["GN","SL","LR"]).tr().ru().ap().h(0.3).into(),
        r(65,"IVO","Ivory Coast-Burkina Faso",38_000_000,&["CI","BF"]).tr().ur().ap().h(0.4).into(),
        r(66,"TOG","Togo-Benin",17_000_000,&["TG","BJ"]).tr().ur().ap().h(0.5).into(),

        // ── Horn of Africa ──
        r(67,"ET","Ethiopia",126_000_000,&["ET"]).tr().ru().ap().h(0.3).into(),
        r(68,"SOM","Somalia-Djibouti",19_000_000,&["SO","DJ"]).ar().ru().ap().h(0.2).fs().into(),
        r(69,"ER","Eritrea",3_700_000,&["ER"]).ar().ru().ap().h(0.3).fs().into(),

        // ── East Africa ──
        r(70,"KE","Kenya",56_000_000,&["KE"]).tr().ur().ap().h(0.5).into(),
        r(71,"TZ","Tanzania",65_000_000,&["TZ"]).tr().ru().ap().h(0.3).into(),
        r(72,"UG","Uganda",48_000_000,&["UG"]).tr().ur().ap().h(0.3).into(),
        r(73,"GLR","Great Lakes",18_000_000,&["RW","BI"]).tr().ur().h(0.3).into(),
        r(74,"MOZ","Mozambique-Malawi",36_000_000,&["MZ","MW"]).tr().ru().ap().h(0.2).into(),

        // ── Sudan ──
        r(75,"SD","Sudan",48_000_000,&["SD","SS"]).ar().ru().ap().h(0.3).fs().into(),

        // ── Central Africa ──
        r(76,"CD","DR Congo",102_000_000,&["CD"]).tr().ru().ap().h(0.1).fs().into(),
        r(77,"CGM","Congo-Cameroon",42_000_000,&["CM","CG","GA","GQ"]).tr().ru().ap().h(0.2).into(),
        r(78,"CAF","Central African Republic-Chad",18_000_000,&["CF","TD"]).tr().ru().ap().h(0.1).fs().into(),
        r(79,"STP","São Tomé",230_000,&["ST"]).tr().ru().i().h(0.5).into(),

        // ── Angola ──
        r(80,"AO","Angola",36_000_000,&["AO"]).tr().ru().ap().h(0.3).into(),

        // ── Southern Africa ──
        r(81,"ZA","South Africa",62_000_000,&["ZA"]).tr().ur().ap().sp().h(2.0).into(),
        r(82,"NAM","Namibia-Botswana",4_500_000,&["NA","BW"]).tr().ru().ap().h(0.5).into(),
        r(83,"ZAM","Zambia-Zimbabwe",32_000_000,&["ZM","ZW"]).tr().ru().ap().h(0.3).into(),
        r(84,"SWZ","Eswatini-Lesotho",3_000_000,&["SZ","LS"]).tr().ru().h(0.5).into(),

        // ── Madagascar ──
        r(85,"MG","Madagascar",30_000_000,&["MG"]).tr().ru().ap().sp().i().h(0.3).into(),

        // ── Central Asia ──
        r(86,"KZ","Kazakhstan",20_000_000,&["KZ"]).ar().ur().ap().h(2.5).au().into(),
        r(87,"UZ","Uzbekistan",35_000_000,&["UZ"]).ar().ur().ap().h(1.0).au().into(),
        r(88,"TKM","Turkmenistan",6_500_000,&["TM"]).ar().ur().ap().h(0.8).au().into(),
        r(89,"KGZ","Kyrgyzstan-Tajikistan",11_000_000,&["KG","TJ"]).ar().ru().ap().h(0.8).au().into(),

        // ── Afghanistan ──
        r(90,"AF","Afghanistan",41_000_000,&["AF"]).ar().ru().ap().h(0.3).fs().into(),

        // ── Mongolia ──
        r(91,"MN","Mongolia",3_400_000,&["MN"]).ac().ru().ap().h(1.5).into(),

        // ── South Asia ──
        r(92,"IN","India",1_450_000_000,&["IN"]).tr().mc().ap().sp().h(0.5).m(0.6).ag(0.7).into(),
        r(93,"PK","Pakistan",240_000_000,&["PK"]).ar().mc().ap().sp().h(0.6).into(),
        r(94,"BD","Bangladesh",175_000_000,&["BD"]).tr().mc().ap().h(0.4).into(),
        r(95,"NPL","Nepal-Bhutan",32_000_000,&["NP","BT"]).tr().ur().h(0.5).into(),
        r(96,"LKA","Sri Lanka-Maldives",23_000_000,&["LK","MV"]).tr().ur().ap().sp().i().h(1.0).into(),

        // ── Southeast Asia ──
        r(97,"ID","Indonesia",280_000_000,&["ID"]).tr().mc().ap().sp().i().h(1.0).into(),
        r(98,"TH","Thailand",72_000_000,&["TH"]).tr().mc().ap().sp().h(2.0).into(),
        r(99,"VN","Vietnam",100_000_000,&["VN"]).tr().mc().ap().sp().h(2.5).au().into(),
        r(100,"PH","Philippines",117_000_000,&["PH"]).tr().mc().ap().sp().i().h(1.0).into(),
        r(101,"MM","Myanmar",55_000_000,&["MM"]).tr().ur().ap().h(0.6).fs().into(),
        r(102,"MY","Malaysia",34_000_000,&["MY"]).tr().ur().ap().sp().h(2.0).into(),
        r(103,"CLM","Cambodia-Laos",25_000_000,&["KH","LA"]).tr().ur().ap().h(0.8).into(),
        r(104,"SGP","Singapore-Brunei-Timor",6_500_000,&["SG","BN","TL"]).tr().ur().ap().sp().i().h(2.0).into(),

        // ── East Asia ──
        r(105,"CN","China",1_425_000_000,&["CN"]).te().mc().ap().sp().h(4.0).au().m(1.0).ag(0.8).into(),
        r(106,"JP","Japan",124_000_000,&["JP"]).te().mc().ap().sp().i().w().h(13.0).m(0.7).into(),
        r(107,"KR","South Korea",52_000_000,&["KR"]).te().mc().ap().sp().w().h(12.0).m(0.6).into(),
        r(108,"KP","North Korea",26_000_000,&["KP"]).te().ur().au().h(1.0).into(),
        r(109,"TW","Taiwan",24_000_000,&["TW"]).tr().ur().ap().sp().i().w().h(6.0).m(0.5).into(),

        // ── Oceania ──
        r(110,"AU","Australia",27_000_000,&["AU"]).ar().ur().ap().sp().i().w().h(8.0).into(),
        r(111,"NZ","New Zealand",5_200_000,&["NZ"]).te().ur().ap().sp().i().w().h(6.0).into(),
        r(112,"PNG","Papua New Guinea",10_000_000,&["PG"]).tr().ru().ap().i().h(0.3).into(),
        r(113,"PAC","Pacific Islands",5_000_000,&["FJ","SB","VU","WS","TO","KI","MH","FM","PW","TV","NR","NC","PF","GU"]).tr().ru().ap().sp().i().h(0.5).into(),

        // ── Greenland ──
        r(114,"GL","Greenland",57_000,&["GL"]).ac().ru().i().h(3.0).into(),
    ]
}

pub fn svg_code_to_region_map(regions: &[Region]) -> HashMap<String, u16> {
    let mut map = HashMap::new();
    for region in regions {
        for code in &region.svg_codes {
            map.insert(code.clone(), region.id);
        }
    }
    map
}
