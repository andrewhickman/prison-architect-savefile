use std::collections::HashMap;

use prison_architect_savefile::Node;

fn main() -> anyhow::Result<()> {
    let mut prison = prison_architect_savefile::read(r#"cloudsaves\felrock2.prison"#)?;

    let programs = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\prison-architect\main\data\reform_programs.txt"#,
    )?;
    let programs_dlc = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\prison-architect\main\data\reform_programs_dlc.txt"#,
    )?;
    let definitions: HashMap<&str, &Node> = programs
        .children()
        .chain(programs_dlc.children())
        .map(|(_, program)| (program.property("Name").unwrap(), program))
        .collect();

    let reform = prison.child_mut("Reform").unwrap();
    let mut programs: Vec<_> = reform
        .child_mut("Programs")
        .unwrap()
        .clear_children()
        .collect();

    programs.sort_unstable_by_key(|(_, program)| program.property("Type").unwrap().to_owned());
    programs.sort_by_key(|(_, program)| {
        program
            .property("StartHour")
            .unwrap()
            .parse::<u32>()
            .unwrap()
    });
    programs.sort_by_key(|(_, program)| {
        let definition = definitions[program.property("Type").unwrap()];
        definition
            .properties()
            .any(|p| p.0 == "Property" && p.1 == "CanHireExternally")
    });

    // programs.retain(|(_, program)| {
    //     program.property("Type") == Some("Methadone")
    //         && program.child("Students").unwrap().children().count() > 0
    // });

    reform.set_property("NextProgramId", programs.len().to_string());
    for (i, (id, program)) in programs.iter_mut().enumerate() {
        *id = format!("[i {}]", i);

        program.set_property("Id", i.to_string());
    }

    reform
        .child_mut("Programs")
        .unwrap()
        .extend_children(programs);

    // prison.set_property("CheatsEnabled", "true");
    // prison
    //     .child_mut("Objects")
    //     .unwrap()
    //     .children_mut()
    //     .filter(|(_, obj)| obj.property("Type") == Some("Tree"))
    //     .for_each(|(_, obj)| obj.set_property("Natural", "true"));

    prison.write(r#"saves\after.prison"#)?;
    Ok(())
}
