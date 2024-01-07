

fn main() -> anyhow::Result<()> {
    let prison = prison_architect_savefile::read(r#"cloudsaves\felrock2.prison"#)?;

    let mut count = 0;
    let mut punishment = 0.0;
    let mut reform = 0.0;
    let mut security = 0.0;
    let mut health = 0.0;
    let mut work = 0.0;
    for (id, prisoner) in prison.child("Victory").unwrap().child("Log").unwrap().children() {
        count += 1;
        punishment += prisoner.property("Grading_Punishment").unwrap().parse::<f64>().unwrap();
        reform += prisoner.property("Grading_Reform").unwrap().parse::<f64>().unwrap();
        security += prisoner.property("Grading_Health").unwrap().parse::<f64>().unwrap();
        health += prisoner.property("Grading_Security").unwrap().parse::<f64>().unwrap();
        work += prisoner.property("Grading_WorkExperience").unwrap_or("0.0").parse::<f64>().unwrap();
    }

    println!("count: {}", count);
    println!("punishment: {}", punishment / count as f64);
    println!("reform: {}", reform / count as f64);
    println!("security: {}", security / count as f64);
    println!("health: {}", health / count as f64);
    println!("work: {}", work / count as f64);

    Ok(())
}