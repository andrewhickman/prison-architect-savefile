use prison_architect_savefile::Node;

fn main() -> anyhow::Result<()> {
    let mut prison = prison_architect_savefile::read(
        r#"C:\Users\andre\Repositories\pa\cloudsaves\felrock2.prison"#,
    )?;

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

    // programs.retain(|(_, program)| {
    //     program.property("Type") == Some("Methadone")
    //         && program.child("Students").unwrap().children().count() > 0
    // });

    reform.set_property("NextProgramId", programs.len().to_string());
    for (i, (id, old_program)) in programs.iter_mut().enumerate() {
        *id = format!("[i {}]", i);

        let mut program = Node::new();
        program.set_property("Id", i.to_string());
        program.set_property("Type", old_program.property("Type").unwrap());
        program.set_property("StartHour", old_program.property("StartHour").unwrap());
        program.set_property("Room.i", old_program.property("Room.i").unwrap());
        program.set_property("Room.u", old_program.property("Room.u").unwrap());
        program.set_property("ManualProgram", "true");
        *old_program = program;
    }

    reform
        .child_mut("Programs")
        .unwrap()
        .extend_children(programs);

    prison.write(r#"C:\Users\andre\Repositories\pa\saves\after.prison"#)?;
    Ok(())
}
