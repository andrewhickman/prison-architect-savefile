use prison_architect_savefile::Node;

#[test]
fn roundtrip() {
    let prison = prison_architect_savefile::read("tests/example.prison").unwrap();
    let serialized = prison.to_string();
    let roundtripped: Node = serialized.parse().unwrap();

    assert_eq!(prison, roundtripped);
}
