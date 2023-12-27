use std::path::PathBuf;

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
    let args = Args::parse();
    let mut prison = prison_architect_savefile::read(args.input)?;

    let electricity = prison.child_mut("Electricity").unwrap();
    let upgraded_wires: Vec<(String, Node)> = electricity
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

    prison.write(args.output)?;

    Ok(())
}
