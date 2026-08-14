use std::collections::HashMap;
use tiny_skia::{Color, Pixmap, Paint, Transform, FillRule};
use usvg::{Tree, Node};

#[derive(Debug, Clone)]
pub struct SvgCountry {
    pub code: String,
    pub name: String,
}

fn preprocess_svg(svg: &str) -> String {
    let lines: Vec<String> = svg.lines().map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with("<path") && !trimmed.contains("id=") && trimmed.contains("class=") {
            if let Some(class_val) = extract_attr(trimmed, "class") {
                let code = class_name_to_code(&class_val);
                let old = format!("class=\"{class_val}\"");
                let new = format!("id=\"{code}\"");
                return line.replace(&old, &new);
            }
        }
        line.to_string()
    }).collect();
    lines.join("\n")
}

pub fn parse_world_svg(svg_content: &str) -> Vec<SvgCountry> {
    let processed = preprocess_svg(svg_content);
    let mut countries: Vec<SvgCountry> = Vec::new();
    let mut seen = HashMap::new();

    for line in processed.lines() {
        let line = line.trim();
        if !line.starts_with("<path") {
            continue;
        }
        if let Some(id) = extract_attr(line, "id") {
            let name = extract_attr(line, "name").unwrap_or_else(|| id.clone());
            if !seen.contains_key(&id) {
                seen.insert(id.clone(), ());
                countries.push(SvgCountry { code: id, name });
            }
        }
    }
    countries
}

fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    if let Some(start) = line.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = line[value_start..].find('"') {
            return Some(line[value_start..value_start + end].to_string());
        }
    }
    None
}

pub fn class_name_to_code(class: &str) -> String {
    match class {
        "United Kingdom" => "GB".to_string(),
        "United States" => "US".to_string(),
        "Dem. Rep. Korea" => "KP".to_string(),
        "Republic of Korea" => "KR".to_string(),
        "Côte d'Ivoire" => "CI".to_string(),
        "Central African Republic" => "CF".to_string(),
        "Dominican Republic" => "DO".to_string(),
        "Czech Republic" => "CZ".to_string(),
        "Bosnia and Herzegovina" => "BA".to_string(),
        "Brunei Darussalam" => "BN".to_string(),
        "Equatorial Guinea" => "GQ".to_string(),
        "French Guiana" => "GF".to_string(),
        "Guinea-Bissau" => "GW".to_string(),
        "Lao PDR" => "LA".to_string(),
        "Saudi Arabia" => "SA".to_string(),
        "South Africa" => "ZA".to_string(),
        "South Sudan" => "SS".to_string(),
        "Sri Lanka" => "LK".to_string(),
        "Timor-Leste" => "TL".to_string(),
        "United Arab Emirates" => "AE".to_string(),
        "Western Sahara" => "EH".to_string(),
        "New Zealand" => "NZ".to_string(),
        "Denmark" => "DK".to_string(),
        "Finland" => "FI".to_string(),
        "Norway" => "NO".to_string(),
        "Sweden" => "SE".to_string(),
        "Azerbaijan" => "AZ".to_string(),
        "Angola" => "AO".to_string(),
        "Chile" => "CL".to_string(),
        "Greece" => "GR".to_string(),
        "Oman" => "OM".to_string(),
        "Cyprus" => "CY".to_string(),
        "Malta" => "MT".to_string(),
        "Fiji" => "FJ".to_string(),
        "Bahamas" => "BS".to_string(),
        "Comoros" => "KM".to_string(),
        "Seychelles" => "SC".to_string(),
        "Mauritius" => "MU".to_string(),
        "Samoa" => "WS".to_string(),
        "Tonga" => "TO".to_string(),
        "Vanuatu" => "VU".to_string(),
        "Russian Federation" => "RU".to_string(),
        "China" => "CN".to_string(),
        "Canada" => "CA".to_string(),
        "Australia" => "AU".to_string(),
        "Argentina" => "AR".to_string(),
        "Brazil" => "BR".to_string(),
        "India" => "IN".to_string(),
        "Russia" => "RU".to_string(),
        _ => {
            let code: String = class
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .take(2)
                .collect::<String>()
                .to_uppercase();
            if code.len() >= 2 { code } else { "XX".to_string() }
        }
    }
}

pub fn rasterize_svg_to_lookup(
    svg_content: &str,
    width: usize,
    height: usize,
) -> (Vec<u16>, Vec<SvgCountry>) {
    let processed = preprocess_svg(svg_content);
    let countries = parse_world_svg(svg_content);

    let code_to_idx: HashMap<String, u16> = countries
        .iter()
        .enumerate()
        .map(|(i, c)| (c.code.clone(), (i + 1) as u16))
        .collect();

    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = Tree::from_str(&processed, &opt).expect("Failed to parse SVG");

    let svg_w = tree.size().width();
    let svg_h = tree.size().height();
    let sx = width as f32 / svg_w;
    let sy = height as f32 / svg_h;

    let mut lookup = vec![0u16; width * height];

    render_group(tree.root(), &code_to_idx, &mut lookup, width, height, sx, sy);

    (lookup, countries)
}

fn render_group(
    group: &usvg::Group,
    code_to_idx: &HashMap<String, u16>,
    lookup: &mut [u16],
    width: usize,
    height: usize,
    sx: f32,
    sy: f32,
) {
    for child in group.children() {
        match child {
            Node::Path(path) => {
                let id = path.id();
                if !id.is_empty() {
                    if let Some(&idx) = code_to_idx.get(id) {
                        let sk_path = path.data();
                        fill_path(sk_path, lookup, width, height, idx, sx, sy);
                    }
                }
            }
            Node::Group(inner) => {
                render_group(&inner, code_to_idx, lookup, width, height, sx, sy);
            }
            _ => {}
        }
    }
}

fn fill_path(
    path: &tiny_skia_path::Path,
    lookup: &mut [u16],
    width: usize,
    height: usize,
    country_id: u16,
    sx: f32,
    sy: f32,
) {
    let mut pixmap = Pixmap::new(width as u32, height as u32).unwrap();

    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(255, 255, 255, 255));

    // Scale the path
    let transform = Transform::from_scale(sx, sy);

    pixmap.fill_path(
        path,
        &paint,
        FillRule::Winding,
        transform,
        None,
    );

    for (i, pixel) in pixmap.pixels().iter().enumerate() {
        if pixel.red() > 128 && lookup[i] == 0 {
            lookup[i] = country_id;
        }
    }
}
