use include_dir::include_dir;
use ron_edit::File;

const INPUTS: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/input");

#[test]
fn ensure_valid_ron() {
    for file in INPUTS.entries() {
        let _: ron::Value = ron::de::from_bytes(file.as_file().unwrap().contents()).unwrap();
    }
}

#[test]
fn format() {
    for file in INPUTS.entries() {
        let contents = file.as_file().unwrap().contents_utf8().unwrap();
        let ron: File = contents.try_into().map_err(|e| eprintln!("{e}")).unwrap();

        let before: ron::Value = ron::de::from_str(contents).unwrap();

        let ron = ronda::format(&ron);

        println!("{ron}");
        let after: ron::Value = ron::de::from_str(&ron).unwrap();

        assert_eq!(before, after);

        insta::assert_snapshot!(file.path().display().to_string(), ron);
    }
}
