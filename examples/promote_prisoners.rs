use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use prison_architect_savefile::Node;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    /// Path to the input prison file.
    input: PathBuf,
    /// Path to write the output prison file.
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let programs = prison_architect_savefile::read(
        r#"C:\Program Files (x86)\Steam\steamapps\common\Prison Architect\main\data\reform_programs.txt"#,
    )?;
    let programs_dlc = prison_architect_savefile::read(
        r#"C:\Program Files (x86)\Steam\steamapps\common\Prison Architect\main\data\reform_programs_dlc.txt"#,
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
        program_scores.insert(program.property("Name").unwrap(), score);
    }

    let mut prison = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\pa\cloudsaves\felrock2.prison"#,
    )?;

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

    let mut min_sec_spaces = 112;
    prisoners.iter_mut().for_each(|(prisoner, remaining, _)| {
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

        score += (experience_total / 0.5).floor() as u32;

        let is_reformed = score >= 10;
        let is_gang_member = prisoner
            .child("Bio")
            .unwrap()
            .properties()
            .any(|(key, value)| key == "Reputation" && value == "GangMember");
        let is_deadly = (prisoner
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
                    .any(|(key, value)| key == "ReputationHigh" && value == "Deadly"));
        let is_addict = prisoner
            .child("Needs")
            .unwrap()
            .child("Needs")
            .unwrap()
            .children()
            .any(|(_, need)| matches!(need.property("Type"), Some("Drugs") | Some("Alcohol")));
        let is_vulnerable = prisoner
            .child("Bio")
            .unwrap()
            .properties()
            .any(|(key, value)| {
                matches!(key, "Reputation" | "ReputationHigh")
                    && matches!(
                        value,
                        "Snitch" | "ExLaw" | "ExPrisonGuard" | "FederalWitness"
                    )
            });

        let category = if is_deadly {
            "SuperMax"
        } else if is_gang_member {
            "MaxSec"
        } else if is_vulnerable {
            "Protected"
        } else if is_addict {
            "MaxSec"
        } else if is_reformed && min_sec_spaces > 0 {
            min_sec_spaces -= 1;
            "MinSec"
        } else {
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

        if *remaining < 120 {
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

    let electricity = prison.child_mut("Electricity").unwrap();
    let upgraded_wires: Vec<_> = electricity
        .children()
        .map(|(key, value)| {
            let mut parts = key.split(' ');
            let x = parts.next().unwrap();
            let y = parts.next().unwrap();
            (format!("{} {} 2", x, y), value.clone())
        })
        .collect();
    electricity.extend_children(upgraded_wires);

    let water = prison.child_mut("Water").unwrap();
    for (_, child) in &mut water.children_mut() {
        if let Some("2") = child.property("PipeType") {
            child.set_property("PipeStatus", "2");
        }
        if let Some("true") = child.property("HotPipe") {
            child.set_property("HotPipeStatus", "2");
        }
    }

    prison.write(r#"C:\Users\andre\Repositories\pa\saves\after.prison"#)?;
    Ok(())
}
