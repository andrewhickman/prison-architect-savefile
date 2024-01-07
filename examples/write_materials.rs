


fn main() -> anyhow::Result<()> {
    let mat1 = prison_architect_savefile::read(r#"C:\Users\andre\Repositories\prison-architect\main\data\materials.txt"#)?;
    let mat2 = prison_architect_savefile::read(r#"C:\Users\andre\Repositories\prison-architect\main\data\materials_dlc.txt"#)?;

    let mut indoor_mats = Vec::new();
    let mut outdoor_mats = Vec::new();

    for (name, node) in mat1.children().chain(mat2.children()) {
        if name == "Material" && node.property("BlockMovement") != Some("true") {
            let (indoor, outdoor )= match node.property("IndoorOutdoor") {
                None => (true, true),
                Some("1") => (false, true),
                Some("2") => (true, false),
                _ => unreachable!(),
            };

            if indoor {
                indoor_mats.push((node.property("Name").unwrap().to_owned(), node.property("MoveCost").unwrap_or("1.0").parse::<f64>().unwrap()));
            }
            if outdoor {
                outdoor_mats.push((node.property("Name").unwrap().to_owned(), node.property("MoveCost").unwrap_or("1.0").parse::<f64>().unwrap()));
            }
        }
    }

    indoor_mats.sort_by_key(|(_, cost)| (cost * 100.0) as u32);
    outdoor_mats.sort_by_key(|(_, cost)| (cost * 100.0) as u32);

    println!("OUTDOOR {:#?}", outdoor_mats);

    Ok(())
}