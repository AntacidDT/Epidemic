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
    pub id: u16,
    pub code: String,
    pub name: String,
    pub svg_codes: Vec<String>,
    pub population: u64,
    pub infected: u64,
    pub dead: u64,
    pub climate: Climate,
    pub density: Density,
    pub is_island: bool,
    pub is_wealthy: bool,
    pub hospital_capacity: f32,
    pub healthcare_collapse: bool,
    pub has_airport: bool,
    pub has_seaport: bool,
    pub borders_open: bool,
    pub air_borders_open: bool,
    pub sea_borders_open: bool,
    pub panic: f32,
    pub government_type: GovernmentType,
    pub misinformation: f32,
    pub lockdown_level: f32,
    pub cure_progress: f32,
    pub vaccine_doses: u64,
    pub vaccinated: u64,
    pub fallen: bool,
    pub manufacturing_capacity: f32,
    pub agricultural_capacity: f32,
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

    pub fn healthy(&self) -> u64 { self.population.saturating_sub(self.infected + self.dead) }
    pub fn infection_pct(&self) -> f32 { if self.population == 0 { 0.0 } else { self.infected as f32 / self.population as f32 } }
    pub fn death_pct(&self) -> f32 { if self.population == 0 { 0.0 } else { self.dead as f32 / self.population as f32 } }
    pub fn is_overwhelmed(&self) -> bool { self.infected as f32 > self.population as f32 * self.hospital_capacity / 1000.0 }
    pub fn mortality_multiplier(&self) -> f32 { if self.healthcare_collapse { 3.0 } else if self.is_overwhelmed() { 1.5 } else { 1.0 } }
    pub fn record_history(&mut self, tick: u64) {
        if self.infection_history.len() >= 200 { self.infection_history.remove(0); }
        if self.death_history.len() >= 200 { self.death_history.remove(0); }
        self.infection_history.push((tick, self.infected));
        self.death_history.push((tick, self.dead));
    }
}

// Builder
struct B(Region);
fn r(id: u16, code: &str, name: &str, pop: u64, codes: &[&str]) -> B { B(Region::new(id, code, name, pop, codes)) }
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
        // ═══════════════════════════════════════
        // NORTH AMERICA
        // ═══════════════════════════════════════
        r(1,"US","United States",341_800_000,&["US"]).w().te().mc().ap().sp().h(5.8).m(0.8).into(),
        r(2,"CA","Canada",41_000_000,&["CA"]).w().ac().ur().ap().sp().h(4.2).into(),
        r(3,"MX","Mexico",130_000_000,&["MX"]).tr().mc().ap().sp().h(2.1).into(),

        // ═══════════════════════════════════════
        // CENTRAL AMERICA & CARIBBEAN
        // ═══════════════════════════════════════
        r(4,"GT","Guatemala",17_600_000,&["GT"]).tr().ur().ap().h(0.6).into(),
        r(5,"BZ","Belize",410_000,&["BZ"]).tr().ru().ap().i().h(1.0).into(),
        r(6,"HN","Honduras",10_400_000,&["HN"]).tr().ur().ap().h(0.7).into(),
        r(7,"SV","El Salvador",6_300_000,&["SV"]).tr().ur().h(1.0).into(),
        r(8,"NI","Nicaragua",6_900_000,&["NI"]).tr().ur().h(0.6).into(),
        r(9,"CR","Costa Rica",5_200_000,&["CR"]).tr().ur().ap().sp().h(1.2).into(),
        r(10,"PA","Panama",4_400_000,&["PA"]).tr().ur().ap().sp().h(1.5).into(),
        r(11,"CU","Cuba",11_300_000,&["CU"]).tr().ur().ap().sp().i().h(1.0).into(),
        r(12,"JM","Jamaica",2_800_000,&["JM"]).tr().ur().ap().sp().i().h(1.0).into(),
        r(13,"HT","Haiti",11_700_000,&["HT"]).tr().ru().ap().h(0.3).fs().into(),
        r(14,"DO","Dominican Republic",11_100_000,&["DO"]).tr().ur().ap().sp().h(0.8).into(),
        r(15,"TT","Trinidad and Tobago",1_500_000,&["TT"]).tr().ur().ap().sp().i().h(1.5).into(),
        r(16,"BS","Bahamas",410_000,&["BS"]).tr().ur().ap().sp().i().h(2.0).into(),
        r(17,"BB","Barbados",280_000,&["BB"]).tr().ur().ap().sp().i().h(2.0).into(),
        r(18,"PR","Puerto Rico",3_200_000,&["PR"]).tr().ur().ap().sp().i().h(2.0).into(),

        // ═══════════════════════════════════════
        // SOUTH AMERICA
        // ═══════════════════════════════════════
        r(19,"BR","Brazil",216_000_000,&["BR"]).tr().mc().ap().sp().h(2.3).m(0.7).ag(0.8).into(),
        r(20,"AR","Argentina",47_000_000,&["AR"]).te().ur().ap().sp().h(3.0).into(),
        r(21,"CO","Colombia",52_000_000,&["CO"]).tr().mc().ap().sp().h(1.8).into(),
        r(22,"PE","Peru",34_000_000,&["PE"]).tr().ur().ap().sp().h(1.6).into(),
        r(23,"VE","Venezuela",28_000_000,&["VE"]).tr().ur().ap().sp().h(1.0).fs().into(),
        r(24,"CL","Chile",19_500_000,&["CL"]).te().ur().ap().sp().h(3.0).into(),
        r(25,"EC","Ecuador",18_000_000,&["EC"]).tr().ur().ap().sp().h(1.4).into(),
        r(26,"BO","Bolivia",12_400_000,&["BO"]).tr().ru().h(1.2).into(),
        r(27,"PY","Paraguay",7_500_000,&["PY"]).te().ur().h(1.5).into(),
        r(28,"UY","Uruguay",3_400_000,&["UY"]).te().ur().ap().sp().h(2.5).into(),
        r(29,"GY","Guyana",810_000,&["GY"]).tr().ru().ap().h(0.8).into(),
        r(30,"SR","Suriname",620_000,&["SR"]).tr().ru().ap().h(0.8).into(),
        r(31,"GF","French Guiana",310_000,&["GF"]).tr().ru().ap().h(1.0).into(),

        // ═══════════════════════════════════════
        // EUROPE — WEST
        // ═══════════════════════════════════════
        r(32,"GB","United Kingdom",69_000_000,&["GB"]).w().te().mc().ap().sp().i().h(5.5).m(0.6).into(),
        r(33,"IE","Ireland",5_300_000,&["IE"]).w().te().ur().ap().sp().i().h(5.0).into(),
        r(34,"FR","France",68_000_000,&["FR"]).w().te().mc().ap().sp().h(6.0).m(0.6).into(),
        r(35,"DE","Germany",84_000_000,&["DE"]).w().te().mc().ap().sp().h(8.0).m(0.8).into(),
        r(36,"ES","Spain",48_000_000,&["ES"]).w().te().ur().ap().sp().h(4.5).into(),
        r(37,"PT","Portugal",10_400_000,&["PT"]).w().te().ur().ap().sp().h(3.5).into(),
        r(38,"IT","Italy",59_000_000,&["IT"]).w().te().mc().ap().sp().h(5.0).into(),
        r(39,"NL","Netherlands",17_800_000,&["NL"]).w().te().ur().ap().sp().h(6.5).into(),
        r(40,"BE","Belgium",11_700_000,&["BE"]).w().te().ur().ap().h(6.0).into(),
        r(41,"LU","Luxembourg",660_000,&["LU"]).w().te().ur().h(8.0).into(),
        r(42,"CH","Switzerland",9_000_000,&["CH"]).w().te().ur().ap().h(8.0).into(),
        r(43,"AT","Austria",9_200_000,&["AT"]).w().te().ur().ap().h(7.0).into(),

        // ═══════════════════════════════════════
        // EUROPE — NORDIC
        // ═══════════════════════════════════════
        r(44,"SE","Sweden",10_500_000,&["SE"]).w().ac().ur().ap().sp().h(5.0).into(),
        r(45,"NO","Norway",5_500_000,&["NO"]).w().ac().ur().ap().sp().h(5.0).into(),
        r(46,"DK","Denmark",5_900_000,&["DK"]).w().te().ur().ap().sp().h(5.5).into(),
        r(47,"FI","Finland",5_600_000,&["FI"]).w().ac().ur().ap().h(4.5).into(),
        r(48,"IS","Iceland",380_000,&["IS"]).w().ac().ur().ap().i().h(4.0).into(),

        // ═══════════════════════════════════════
        // EUROPE — CENTRAL
        // ═══════════════════════════════════════
        r(49,"PL","Poland",38_000_000,&["PL"]).te().mc().ap().h(3.5).into(),
        r(50,"CZ","Czech Republic",10_900_000,&["CZ"]).te().ur().ap().h(4.0).into(),
        r(51,"SK","Slovakia",5_500_000,&["SK"]).te().ur().ap().h(3.5).into(),
        r(52,"HU","Hungary",9_700_000,&["HU"]).te().ur().ap().h(3.5).into(),

        // ═══════════════════════════════════════
        // EUROPE — EAST
        // ═══════════════════════════════════════
        r(53,"UA","Ukraine",37_000_000,&["UA"]).te().mc().ap().h(2.5).au().into(),
        r(54,"BY","Belarus",9_200_000,&["BY"]).te().ur().ap().h(2.5).au().into(),
        r(55,"MD","Moldova",2_600_000,&["MD"]).te().ur().h(2.0).into(),
        r(56,"RO","Romania",19_000_000,&["RO"]).te().ur().ap().h(3.0).into(),
        r(57,"BG","Bulgaria",6_500_000,&["BG"]).te().ur().ap().h(3.0).into(),

        // ═══════════════════════════════════════
        // EUROPE — BALKANS
        // ═══════════════════════════════════════
        r(58,"HR","Croatia",3_900_000,&["HR"]).te().ur().ap().h(3.5).into(),
        r(59,"RS","Serbia",6_600_000,&["RS"]).te().ur().ap().h(3.0).into(),
        r(60,"BA","Bosnia and Herzegovina",3_200_000,&["BA"]).te().ur().ap().h(2.5).into(),
        r(61,"SI","Slovenia",2_100_000,&["SI"]).w().te().ur().ap().h(4.5).into(),
        r(62,"AL","Albania",2_800_000,&["AL"]).te().ur().ap().h(2.0).into(),
        r(63,"XK","Kosovo",1_800_000,&["XK"]).te().ur().h(2.0).into(),
        r(64,"MK","North Macedonia",1_800_000,&["MK"]).te().ur().ap().h(2.5).into(),
        r(65,"ME","Montenegro",620_000,&["ME"]).te().ur().h(2.5).into(),
        r(66,"GR","Greece",10_400_000,&["GR"]).te().ur().ap().sp().h(3.0).into(),

        // ═══════════════════════════════════════
        // BALTICS
        // ═══════════════════════════════════════
        r(67,"EE","Estonia",1_400_000,&["EE"]).te().ur().ap().h(4.0).into(),
        r(68,"LV","Latvia",1_800_000,&["LV"]).te().ur().ap().h(3.5).into(),
        r(69,"LT","Lithuania",2_800_000,&["LT"]).te().ur().ap().h(3.5).into(),

        // ═══════════════════════════════════════
        // RUSSIA
        // ═══════════════════════════════════════
        r(70,"RU","Russia",144_000_000,&["RU","Russian Federation"]).ac().mc().ap().sp().h(4.0).au().m(0.5).into(),

        // ═══════════════════════════════════════
        // CAUCASUS
        // ═══════════════════════════════════════
        r(71,"GE","Georgia",3_700_000,&["GE"]).te().ur().ap().h(2.0).into(),
        r(72,"AM","Armenia",2_800_000,&["AM"]).te().ur().ap().h(2.0).into(),
        r(73,"AZ","Azerbaijan",10_200_000,&["AZ"]).te().ur().ap().h(2.0).into(),

        // ═══════════════════════════════════════
        // MIDDLE EAST
        // ═══════════════════════════════════════
        r(74,"TR","Turkey",86_000_000,&["Turkey"]).ar().mc().ap().sp().h(2.8).into(),
        r(75,"IR","Iran",88_000_000,&["IR","Iran"]).ar().mc().ap().sp().h(1.6).au().into(),
        r(76,"IQ","Iraq",43_000_000,&["IQ","Iraq"]).ar().ur().ap().h(1.0).fs().into(),
        r(77,"SA","Saudi Arabia",37_000_000,&["SA","Saudi Arabia"]).ar().ur().ap().sp().w().h(2.5).into(),
        r(78,"AE","United Arab Emirates",10_000_000,&["AE","United Arab Emirates"]).ar().ur().ap().sp().w().h(3.0).into(),
        r(79,"IL","Israel",9_800_000,&["IL","Israel"]).ar().ur().ap().sp().w().h(5.0).into(),
        r(80,"JO","Jordan",11_500_000,&["JO","Jordan"]).ar().ur().ap().h(2.0).into(),
        r(81,"LB","Lebanon",5_500_000,&["LB","Lebanon"]).ar().ur().ap().h(2.0).into(),
        r(82,"SY","Syria",22_000_000,&["SY","Syria"]).ar().ur().ap().h(0.8).fs().into(),
        r(83,"YE","Yemen",34_000_000,&["YE","Yemen"]).ar().ru().ap().h(0.5).fs().into(),
        r(84,"OM","Oman",4_600_000,&["OM","Oman"]).ar().ur().ap().sp().h(2.0).into(),
        r(85,"QA","Qatar",2_700_000,&["QA","Qatar"]).ar().ur().ap().sp().w().h(3.0).into(),
        r(86,"BH","Bahrain",1_500_000,&["BH","Bahrain"]).ar().ur().ap().sp().w().h(3.0).into(),
        r(87,"KW","Kuwait",4_300_000,&["KW","Kuwait"]).ar().ur().ap().sp().w().h(2.5).into(),
        r(88,"PS","Palestine",5_400_000,&["PS","Palestine"]).ar().ur().h(1.0).into(),

        // ═══════════════════════════════════════
        // NORTH AFRICA
        // ═══════════════════════════════════════
        r(89,"EG","Egypt",106_000_000,&["EG","Egypt"]).ar().mc().ap().sp().h(1.6).au().into(),
        r(90,"LY","Libya",7_000_000,&["LY","Libya"]).ar().ur().ap().h(0.8).au().into(),
        r(91,"DZ","Algeria",46_000_000,&["DZ","Algeria"]).ar().ur().ap().sp().h(1.5).au().into(),
        r(92,"MA","Morocco",37_500_000,&["MA","Morocco"]).ar().ur().ap().sp().h(1.2).into(),
        r(93,"TN","Tunisia",12_000_000,&["TN","Tunisia"]).ar().ur().ap().sp().h(1.5).into(),
        r(94,"EH","Western Sahara",600_000,&["EH","Western Sahara"]).ar().ru().h(0.3).into(),

        // ═══════════════════════════════════════
        // WEST AFRICA
        // ═══════════════════════════════════════
        r(95,"NG","Nigeria",224_000_000,&["NG","Nigeria"]).tr().mc().ap().sp().h(0.5).into(),
        r(96,"GH","Ghana",34_000_000,&["GH","Ghana"]).tr().ur().ap().h(0.8).into(),
        r(97,"SN","Senegal",18_000_000,&["SN","Senegal"]).tr().ur().ap().sp().h(0.5).into(),
        r(98,"ML","Mali",22_000_000,&["ML","Mali"]).tr().ru().ap().h(0.2).into(),
        r(99,"BF","Burkina Faso",22_000_000,&["BF","Burkina Faso"]).tr().ru().ap().h(0.2).into(),
        r(100,"NE","Niger",26_000_000,&["NE","Niger"]).tr().ru().ap().h(0.2).into(),
        r(101,"CI","Côte d'Ivoire",28_000_000,&["CI"]).tr().ur().ap().h(0.4).into(),
        r(102,"GN","Guinea",14_000_000,&["GN","Guinea"]).tr().ru().ap().h(0.3).into(),
        r(103,"SL","Sierra Leone",8_600_000,&["SL","Sierra Leone"]).tr().ru().ap().h(0.2).into(),
        r(104,"LR","Liberia",5_300_000,&["LR","Liberia"]).tr().ru().ap().h(0.2).into(),
        r(105,"BJ","Benin",13_000_000,&["BJ","Benin"]).tr().ur().ap().h(0.5).into(),
        r(106,"TG","Togo",8_800_000,&["TG","Togo"]).tr().ur().ap().h(0.5).into(),
        r(107,"MR","Mauritania",4_800_000,&["MR","Mauritania"]).ar().ru().ap().h(0.3).into(),
        r(108,"GM","Gambia",2_700_000,&["GM","Gambia"]).tr().ru().h(0.3).into(),
        r(109,"GW","Guinea-Bissau",2_100_000,&["GW","Guinea-Bissau"]).tr().ru().h(0.2).into(),
        r(110,"CV","Cape Verde",600_000,&["CV","Cape Verde"]).tr().ur().ap().sp().i().h(1.0).into(),

        // ═══════════════════════════════════════
        // SUDAN
        // ═══════════════════════════════════════
        r(111,"SD","Sudan",48_000_000,&["SD"]).ar().ru().ap().h(0.3).fs().into(),
        r(112,"SS","South Sudan",11_000_000,&["SS"]).tr().ru().h(0.1).fs().into(),

        // ═══════════════════════════════════════
        // EAST AFRICA
        // ═══════════════════════════════════════
        r(115,"ET","Ethiopia",126_000_000,&["ET","Ethiopia"]).tr().ru().ap().h(0.3).into(),
        r(112,"KE","Kenya",56_000_000,&["KE","Kenya"]).tr().ur().ap().h(0.5).into(),
        r(115,"TZ","Tanzania",65_000_000,&["TZ","Tanzania"]).tr().ru().ap().h(0.3).into(),
        r(116,"UG","Uganda",48_000_000,&["UG","Uganda"]).tr().ur().ap().h(0.3).into(),
        r(117,"RW","Rwanda",14_000_000,&["RW","Rwanda"]).tr().ur().h(0.5).into(),
        r(118,"BI","Burundi",13_000_000,&["BI","Burundi"]).tr().ru().h(0.2).into(),
        r(119,"DJ","Djibouti",1_100_000,&["DJ","Djibouti"]).ar().ur().ap().sp().h(0.5).into(),
        r(120,"ER","Eritrea",3_700_000,&["ER","Eritrea"]).ar().ru().ap().h(0.3).fs().into(),
        r(121,"SO","Somalia",18_000_000,&["SO","Somalia"]).ar().ru().ap().h(0.2).fs().into(),
        r(122,"MG","Madagascar",30_000_000,&["MG","Madagascar"]).tr().ru().ap().sp().i().h(0.3).into(),
        r(123,"MZ","Mozambique",33_000_000,&["MZ","Mozambique"]).tr().ru().ap().sp().h(0.2).into(),
        r(124,"MW","Malawi",20_000_000,&["MW","Malawi"]).tr().ru().h(0.2).into(),
        r(125,"ZM","Zambia",20_000_000,&["ZM","Zambia"]).tr().ru().ap().h(0.3).into(),
        r(126,"ZW","Zimbabwe",16_000_000,&["ZW","Zimbabwe"]).tr().ru().ap().h(0.3).into(),

        // ═══════════════════════════════════════
        // CENTRAL AFRICA
        // ═══════════════════════════════════════
        r(127,"CD","DR Congo",102_000_000,&["CD"]).tr().ru().ap().h(0.1).fs().into(),
        r(128,"CG","Republic of Congo",6_000_000,&["CG"]).tr().ru().ap().h(0.2).into(),
        r(129,"CM","Cameroon",28_000_000,&["CM","Cameroon"]).tr().ur().ap().h(0.4).into(),
        r(130,"GA","Gabon",2_400_000,&["GA","Gabon"]).tr().ur().ap().h(0.8).into(),
        r(131,"GQ","Equatorial Guinea",1_700_000,&["GQ","Equatorial Guinea"]).tr().ur().h(0.5).into(),
        r(132,"TD","Chad",17_000_000,&["TD","Chad"]).ar().ru().ap().h(0.1).fs().into(),
        r(133,"CF","Central African Republic",5_000_000,&["CF"]).tr().ru().h(0.1).fs().into(),
        r(134,"AO","Angola",36_000_000,&["AO"]).tr().ru().ap().h(0.3).into(),
        r(135,"ST","São Tomé and Principe",230_000,&["São Tomé and Principe"]).tr().ru().i().h(0.5).into(),

        // ═══════════════════════════════════════
        // SOUTHERN AFRICA
        // ═══════════════════════════════════════
        r(136,"ZA","South Africa",62_000_000,&["ZA","South Africa"]).tr().ur().ap().sp().h(2.0).into(),
        r(137,"NA","Namibia",2_600_000,&["NA","Namibia"]).ar().ru().ap().h(0.5).into(),
        r(138,"BW","Botswana",2_600_000,&["BW","Botswana"]).ar().ru().ap().h(0.5).into(),
        r(139,"SZ","Eswatini",1_200_000,&["SZ","Eswatini"]).tr().ru().h(0.5).into(),
        r(140,"LS","Lesotho",2_300_000,&["LS","Lesotho"]).te().ru().h(0.3).into(),

        // ═══════════════════════════════════════
        // CENTRAL ASIA
        // ═══════════════════════════════════════
        r(141,"KZ","Kazakhstan",20_000_000,&["KZ","Kazakhstan"]).ar().ur().ap().h(2.5).au().into(),
        r(142,"UZ","Uzbekistan",35_000_000,&["UZ","Uzbekistan"]).ar().ur().ap().h(1.0).au().into(),
        r(143,"TM","Turkmenistan",6_500_000,&["TM","Turkmenistan"]).ar().ur().ap().h(0.8).au().into(),
        r(144,"KG","Kyrgyzstan",7_000_000,&["KG","Kyrgyzstan"]).ar().ru().ap().h(0.8).into(),
        r(145,"TJ","Tajikistan",10_000_000,&["TJ","Tajikistan"]).ar().ru().ap().h(0.6).into(),

        // ═══════════════════════════════════════
        // AFGHANISTAN
        // ═══════════════════════════════════════
        r(146,"AF","Afghanistan",41_000_000,&["AF","Afghanistan"]).ar().ru().ap().h(0.3).fs().into(),

        // ═══════════════════════════════════════
        // SOUTH ASIA
        // ═══════════════════════════════════════
        r(147,"IN","India",1_450_000_000,&["IN","India"]).tr().mc().ap().sp().h(0.5).m(0.6).ag(0.7).into(),
        r(148,"PK","Pakistan",240_000_000,&["PK","Pakistan"]).ar().mc().ap().sp().h(0.6).into(),
        r(149,"BD","Bangladesh",175_000_000,&["BD","Bangladesh"]).tr().mc().ap().h(0.4).into(),
        r(150,"NP","Nepal",30_000_000,&["NP","Nepal"]).te().ur().h(0.5).into(),
        r(151,"LK","Sri Lanka",22_000_000,&["LK","Sri Lanka"]).tr().ur().ap().sp().i().h(1.0).into(),
        r(152,"BT","Bhutan",780_000,&["BT","Bhutan"]).te().ru().h(0.5).into(),
        r(153,"MV","Maldives",520_000,&["MV","Maldives"]).tr().ur().ap().sp().i().h(1.5).into(),

        // ═══════════════════════════════════════
        // SOUTHEAST ASIA
        // ═══════════════════════════════════════
        r(154,"ID","Indonesia",280_000_000,&["ID","Indonesia"]).tr().mc().ap().sp().i().h(1.0).into(),
        r(155,"TH","Thailand",72_000_000,&["TH","Thailand"]).tr().mc().ap().sp().h(2.0).into(),
        r(156,"VN","Vietnam",100_000_000,&["VN","Vietnam"]).tr().mc().ap().sp().h(2.5).au().into(),
        r(157,"PH","Philippines",117_000_000,&["PH","Philippines"]).tr().mc().ap().sp().i().h(1.0).into(),
        r(158,"MM","Myanmar",55_000_000,&["MM","Myanmar"]).tr().ur().ap().h(0.6).fs().into(),
        r(159,"MY","Malaysia",34_000_000,&["MY","Malaysia"]).tr().ur().ap().sp().h(2.0).into(),
        r(160,"KH","Cambodia",17_000_000,&["KH","Cambodia"]).tr().ur().ap().h(0.8).into(),
        r(161,"LA","Laos",7_500_000,&["LA","Lao PDR"]).tr().ru().h(0.5).into(),
        r(162,"SG","Singapore",5_900_000,&["SG","Singapore"]).tr().ur().ap().sp().i().w().h(5.0).m(0.5).into(),
        r(163,"BN","Brunei",450_000,&["BN","Brunei Darussalam"]).tr().ur().ap().sp().i().w().h(3.0).into(),
        r(164,"TL","Timor-Leste",1_300_000,&["TL","Timor-Leste"]).tr().ru().ap().i().h(0.5).into(),

        // ═══════════════════════════════════════
        // EAST ASIA
        // ═══════════════════════════════════════
        r(165,"CN","China",1_425_000_000,&["CN","China"]).te().mc().ap().sp().h(4.0).au().m(1.0).ag(0.8).into(),
        r(166,"JP","Japan",124_000_000,&["JP","Japan"]).te().mc().ap().sp().i().w().h(13.0).m(0.7).into(),
        r(167,"KR","South Korea",52_000_000,&["KR","Republic of Korea"]).te().mc().ap().sp().w().h(12.0).m(0.6).into(),
        r(168,"KP","North Korea",26_000_000,&["KP","Dem. Rep. Korea"]).te().ur().au().h(1.0).into(),
        r(169,"TW","Taiwan",24_000_000,&["TW","Taiwan"]).tr().ur().ap().sp().i().w().h(6.0).m(0.5).into(),
        r(170,"MN","Mongolia",3_400_000,&["MN","Mongolia"]).ac().ru().ap().h(1.5).into(),

        // ═══════════════════════════════════════
        // OCEANIA
        // ═══════════════════════════════════════
        r(171,"AU","Australia",27_000_000,&["AU","Australia"]).ar().ur().ap().sp().i().w().h(8.0).into(),
        r(172,"NZ","New Zealand",5_200_000,&["NZ","New Zealand"]).te().ur().ap().sp().i().w().h(6.0).into(),
        r(173,"PG","Papua New Guinea",10_000_000,&["PG","Papua New Guinea"]).tr().ru().ap().i().h(0.3).into(),
        r(174,"FJ","Fiji",930_000,&["FJ","Fiji"]).tr().ur().ap().sp().i().h(1.0).into(),
        r(175,"SB","Solomon Islands",720_000,&["SB","Solomon Islands"]).tr().ru().ap().i().h(0.3).into(),
        r(176,"VU","Vanuatu",330_000,&["VU","Vanuatu"]).tr().ru().ap().i().h(0.5).into(),
        r(177,"WS","Samoa",220_000,&["WS","Samoa"]).tr().ru().ap().i().h(0.5).into(),
        r(178,"TO","Tonga",100_000,&["TO","Tonga"]).tr().ru().ap().i().h(0.5).into(),
        r(179,"TV","Tuvalu",11_000,&["TV","Tuvalu"]).tr().ru().i().h(0.3).into(),
        r(180,"NR","Nauru",13_000,&["NR","Nauru"]).tr().ru().i().h(0.3).into(),
        r(181,"PW","Palau",18_000,&["PW","Palau"]).tr().ru().i().h(0.5).into(),
        r(182,"MH","Marshall Islands",42_000,&["MH","Marshall Islands"]).tr().ru().i().h(0.5).into(),
        r(183,"FM","Micronesia",110_000,&["FM","Federated States of Micronesia"]).tr().ru().i().h(0.5).into(),
        r(184,"KI","Kiribati",130_000,&["KI","Kiribati"]).tr().ru().i().h(0.3).into(),
        r(185,"NC","New Caledonia",290_000,&["NC","New Caledonia"]).tr().ur().ap().sp().i().h(1.5).into(),
        r(186,"PF","French Polynesia",280_000,&["PF","French Polynesia"]).tr().ur().ap().sp().i().h(1.0).into(),
        r(187,"GU","Guam",170_000,&["GU","Guam"]).tr().ur().ap().i().h(1.5).into(),

        // ═══════════════════════════════════════
        // GREENLAND
        // ═══════════════════════════════════════
        r(188,"GL","Greenland",57_000,&["GL","Greenland"]).ac().ru().i().h(3.0).into(),
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
