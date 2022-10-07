use std::path::PathBuf;
use std::{fs, process};

#[test]
fn assemble_files() {
    let tests_folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_sas"));
    let tmp_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("assemble_test");

    fs::create_dir_all(&tmp_dir).unwrap();

    let sources: Vec<PathBuf> = fs::read_dir(tests_folder)
        .unwrap()
        .flatten()
        .filter(|f| f.file_name().to_string_lossy().ends_with(".S"))
        .map(|f| f.path())
        .collect();

    for source in sources {
        let output = tmp_dir
            .join(source.file_name().unwrap())
            .with_extension("out");
        let result = process::Command::new(&executable)
            .args([
                "-i",
                &source.to_string_lossy(),
                "-o",
                &output.to_string_lossy(),
                "--data-section-start",
                "0x8000",
            ])
            .output()
            .unwrap();

        assert!(result.status.success());
        assert!(output.exists());
    }
}
