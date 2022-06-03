use std::process::Command;

fn main() {
    Command::new("esbuild")
        .args(&[
            "assets/js/app.js",
            "--bundle",
            "--target=es2017",
            "--outdir=./priv/static/assets",
        ])
        .status()
        .expect("esbuild failed to run");
}
