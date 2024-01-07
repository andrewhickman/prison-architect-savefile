use prison_architect_savefile::Node;

fn main() -> anyhow::Result<()> {
    let mut checkpoint1 = prison_architect_savefile::read(r#"cloudsaves\checkpoint1.prison"#)?;
    let mut checkpoint2 = prison_architect_savefile::read(r#"cloudsaves\checkpoint2.prison"#)?;

    let programs1 = checkpoint1
        .child_mut("Reform")
        .unwrap()
        .child_mut("Programs")
        .unwrap()
        .children();
    let programs2 = checkpoint2
        .child_mut("Reform")
        .unwrap()
        .child_mut("Programs")
        .unwrap()
        .children();

    for ((id, l), (id2, r)) in programs1.zip(programs2) {
        assert_eq!(id, id2);
        if l.property("StartHour") == Some("2") {
            if l.property("ClassProgress") != r.property("ClassProgress") {
                println!("OK  {} {}", id, l.property("Type").unwrap());
            } else {
                println!("ERR {} {}", id, l.property("Type").unwrap());
            }
        }
    }
    Ok(())
}
