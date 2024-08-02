use std::process::Command;

fn main() {
    let dylint_output = Command::new("cargo")
        .args([
            "dylint",
            "--all",
            "--",
            "--all-targets",
            "--message-format=json",
        ])
        .output()
        .unwrap();

    println!("{}", String::from_utf8_lossy(&dylint_output.stdout));

    let clippy_output = Command::new("cargo")
        .args(["clippy", "--all-targets", "--message-format=json"])
        .output()
        .unwrap();

    println!("{}", String::from_utf8_lossy(&clippy_output.stdout));
}
