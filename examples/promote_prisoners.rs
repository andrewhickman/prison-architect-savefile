use std::collections::HashMap;

use anyhow::Context;
use prison_architect_savefile::Node;

fn main() -> anyhow::Result<()> {
    let programs = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\prison-architect\main\data\reform_programs.txt"#,
    )?;
    let programs_dlc = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\prison-architect\main\data\reform_programs_dlc.txt"#,
    )?;
    let mut program_scores = HashMap::new();
    for (_, program) in programs.children().chain(programs_dlc.children()) {
        let difficulty: u32 = program
            .property("Difficulty")
            .unwrap_or("0")
            .parse()
            .context("error parsing difficulty")?;
        let score = if difficulty <= 25 {
            2
        } else if difficulty <= 50 {
            4
        } else {
            6
        };
        program_scores.insert(program.property("Name").unwrap().to_owned(), score);
    }

    let mut prison = prison_architect_savefile::read(r#"cloudsaves\felrock2.prison"#)?;

    let mut prisoners: Vec<(&mut Node, u32, u32)> = prison
        .child_mut("Objects")
        .unwrap()
        .children_mut()
        .filter(|(_, object)| object.property("Type") == Some("Prisoner"))
        .map(|(_, prisoner)| {
            let bio = prisoner.child("Bio").unwrap();
            let sentence: f64 = bio.property("SentenceF").unwrap().parse().unwrap();
            let served: f64 = bio.property("Served").unwrap().parse().unwrap();
            let remaining_hours = ((sentence - served) * 5.0 * 24.0) as u32;

            let time_until_parole = match bio.property("NextParole") {
                Some("Half") => sentence * 0.5,
                Some("ThreeQuarters") => sentence * 0.75,
                _ => sentence,
            };
            let remaining_parole = ((time_until_parole - served) * 5.0 * 24.0) as u32;

            (prisoner, remaining_hours, remaining_parole)
        })
        .collect();
    prisoners.sort_by_key(|(_, _, rem)| *rem);

    let total_drug_addicts = prisoners
        .iter()
        .filter(|(prisoner, _, _)| {
            !is_insane(prisoner)
                // && !is_death_row(prisoner)
                && !is_very_deadly(prisoner)
                && !is_gang_member(prisoner)
                && !is_vulnerable(prisoner)
                && is_drug_addict(prisoner)
        })
        .count();
    let total_alcohol_addicts = prisoners
        .iter()
        .filter(|(prisoner, _, _)| {
            !is_insane(prisoner)
                // && !is_death_row(prisoner)
                && !is_very_deadly(prisoner)
                && !is_gang_member(prisoner)
                && !is_vulnerable(prisoner)
                && is_alcohol_addict(prisoner)
        })
        .count();
    println!("{} drug addicts", total_drug_addicts);
    println!("{} alcohol addicts", total_alcohol_addicts);

    let mut min_sec_spaces = 112;
    let mut drug_spaces = (total_drug_addicts as u32).min(16);
    let mut alcohol_spaces = 16u32 - drug_spaces;
    prisoners.iter_mut().for_each(|(prisoner, remaining, until_parole)| {
        let category = if is_insane(prisoner) {
            "Insane"
        // } else if is_death_row(prisoner) {
        //     "DeathRow"
        } else if is_very_deadly(prisoner) {
            "SuperMax"
        } else if is_vulnerable(prisoner) {
            "Protected"
        } else if is_gang_member(prisoner) {
            "MaxSec"
        } else if is_drug_addict(prisoner) && drug_spaces > 0 {
            drug_spaces -= 1;
            "MaxSec"
        } else if is_alcohol_addict(prisoner) && alcohol_spaces > 0 {
            alcohol_spaces -= 1;
            "MaxSec"
        } else if is_reformed(prisoner, &program_scores) && min_sec_spaces > 0 {
            min_sec_spaces -= 1;
            "MinSec"
        } else {
            if is_reformed(prisoner, &program_scores) {
                println!(
                    "no space for {}",
                    prisoner
                        .child("Bio")
                        .unwrap()
                        .property("Surname")
                        .unwrap_or_else(|| prisoner
                            .child("Bio")
                            .unwrap()
                            .property("Forname")
                            .unwrap()),
                );
            }
            "Normal"
        };

        if prisoner.property("Category") != Some(category) {
            println!(
                "Move {} ({}) from {} to {}",
                prisoner
                    .child("Bio")
                    .unwrap()
                    .property("Surname")
                    .unwrap_or_else(|| prisoner.child("Bio").unwrap().property("Forname").unwrap()),
                prisoner.property("Id.i").unwrap(),
                prisoner.property("Category").unwrap(),
                category,
            );
            prisoner.set_property("Category", category);
        }

        if is_deadly(prisoner) {
            prisoner.set_property("MarkType", "1");
        } else {
            let _ = prisoner.clear_property("MarkType");
        }

        if *until_parole > 6000 {
            println!(
                "{} prisoner {} ({}) due for release in {} hours",
                prisoner.property("Category").unwrap(),
                prisoner
                    .child("Bio")
                    .unwrap()
                    .property("Surname")
                    .unwrap_or_else(|| prisoner.child("Bio").unwrap().property("Forname").unwrap()),
                prisoner.property("Id.u").unwrap(),
                remaining,
            );
        }
    });

    prison
        .child_mut("Objects")
        .unwrap()
        .children_mut()
        .filter(|(_, object)| object.property("Type") == Some("Guard"))
        .for_each(|(id, guard)| {
            if guard.property("Corrupted") == Some("true")
                && guard.property("Exposed") != Some("true")
            {
                println!(
                    "exposed corrupted guard {} ({})",
                    guard.child("Bio").unwrap().property("Name").unwrap(),
                    id
                );
                guard.set_property("Exposed", "true");
            }
        });

    let water = prison.child_mut("Water").unwrap();
    for (_, child) in &mut water.children_mut() {
        if let Some("2") = child.property("PipeType") {
            // child.set_property("PipeStatus", "2");
        }
        if let Some("true") = child.property("HotPipe") {
            // child.set_property("HotPipeStatus", "2");
        }
    }

    prison.write(r#"saves\after.prison"#)?;
    Ok(())
}

fn is_insane(prisoner: &Node) -> bool {
    prisoner.property("Category") == Some("Insane")
}

fn is_death_row(prisoner: &Node) -> bool {
    prisoner.property("Category") == Some("DeathRow")
}

fn is_reformed(prisoner: &Node, program_scores: &HashMap<String, u32>) -> bool {
    let mut score = 0;

    for (name, program) in prisoner
        .child("Experience")
        .unwrap()
        .child("Results")
        .unwrap()
        .children()
    {
        if let Some(count) = program.property("Passed") {
            let count: u32 = count.parse().unwrap();
            score += count * program_scores[name];
        }
    }

    let work_areas = [
        "WorkCook",
        "WorkCleaner",
        "WorkCraftsman",
        "WorkLabourer",
        "WorkRCS",
    ];
    let experience = prisoner
        .child("Experience")
        .unwrap()
        .child("Experience")
        .unwrap();
    let mut experience_total: f64 = 0.0;
    for area in work_areas {
        if let Some(exp) = experience.property(area) {
            let exp: f64 = exp.parse().unwrap();
            experience_total += exp;
        }
    }

    (score + (experience_total / 0.5).floor() as u32) >= 10
}

fn is_gang_member(prisoner: &Node) -> bool {
    prisoner
        .child("Bio")
        .unwrap()
        .properties()
        .any(|(key, value)| key == "Reputation" && value == "GangMember")
}

fn is_very_deadly(prisoner: &Node) -> bool {
    (prisoner
        .child("Bio")
        .unwrap()
        .properties()
        .any(|(key, value)| key == "ReputationHigh" && value == "Volatile")
        && prisoner
            .child("Bio")
            .unwrap()
            .properties()
            .any(|(key, value)| {
                matches!(key, "Reputation" | "ReputationHigh") && value == "Deadly"
            }))
        || (prisoner
            .child("Bio")
            .unwrap()
            .properties()
            .any(|(key, value)| {
                matches!(key, "Reputation" | "ReputationHigh") && value == "Volatile"
            })
            && prisoner
                .child("Bio")
                .unwrap()
                .properties()
                .any(|(key, value)| key == "ReputationHigh" && matches!(value, "Deadly" | "Strong")))
}

fn is_deadly(prisoner: &Node) -> bool {
    prisoner
        .child("Bio")
        .unwrap()
        .properties()
        .any(|(key, value)| matches!(key, "Reputation" | "ReputationHigh") && matches!(value, "Deadly"))
}

fn is_drug_addict(prisoner: &Node) -> bool {
    prisoner
        .child("Needs")
        .unwrap()
        .child("Needs")
        .unwrap()
        .children()
        .any(|(_, need)| matches!(need.property("Type"), Some("Drugs")))
}

fn is_alcohol_addict(prisoner: &Node) -> bool {
    prisoner
        .child("Needs")
        .unwrap()
        .child("Needs")
        .unwrap()
        .children()
        .any(|(_, need)| matches!(need.property("Type"), Some("Alcohol")))
}

fn is_vulnerable(prisoner: &Node) -> bool {
    prisoner
        .child("Bio")
        .unwrap()
        .properties()
        .any(|(key, value)| {
            matches!(key, "Reputation" | "ReputationHigh")
                && matches!(
                    value,
                    "Snitch" | "ExLaw" | "ExPrisonGuard" | "FederalWitness"
                )
        })
}
