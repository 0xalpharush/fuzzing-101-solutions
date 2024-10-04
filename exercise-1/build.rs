use std::{
    borrow::BorrowMut,
    env,
    process::Command,
    str,
};

use anyhow::{anyhow, Context};

fn exec(mut command: impl BorrowMut<Command>) -> anyhow::Result<String> {
    let command = command.borrow_mut();

    let output = command
        .output()
        .with_context(|| anyhow!("Failed to execute command: {command:?}"))?;

    let stdout = str::from_utf8(&output.stdout).unwrap_or("Invalid UTF-8");

    if !output.status.success() {
        let stderr = str::from_utf8(&output.stderr).unwrap_or("Invalid UTF-8");

        eprintln!("Error from {command:?}");
        eprintln!();
        eprintln!("stdout:");
        eprintln!();
        eprintln!("{stdout}");
        eprintln!();
        eprintln!("-------");
        eprintln!("stderr:");
        eprintln!();
        eprintln!("{stderr}");
        eprintln!();
        eprintln!("-------");

        return Err(anyhow!("Failed to execute command: {command:?}")).with_context(|| {
            anyhow!(
                "Command exited with a non-zero exit code: {}",
                output.status
            )
        });
    }

    Ok(stdout.to_string())
}

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");

    let cwd = env::current_dir().unwrap().to_string_lossy().to_string();
    let xpdf_dir = format!("{}/xpdf", cwd);

    // // make clean; remove any leftover gunk from prior builds
    // Command::new("make")
    //     .arg("clean")
    //     .current_dir(xpdf_dir.clone())
    //     .status()
    //     .expect("Couldn't clean xpdf directory");

    // clean doesn't know about the install directory we use to build, remove it as well
    Command::new("rm")
        .arg("-r")
        .arg("-v")
        .arg("-f")
        .arg(&format!("{}/install", xpdf_dir))
        .current_dir(xpdf_dir.clone())
        .status()
        .expect("Couldn't clean xpdf's install directory");

    // export LLVM_CONFIG=llvm-config
    env::set_var("LLVM_CONFIG", "llvm-config");
    // export AFL_PATH=/Users/troysargent/AFLplusplus\
    let afl_path = "/usr/local/lib/afl";
    env::set_var("AFL_PATH",afl_path);


    // configure with afl-clang-fast and set install directory to ./xpdf/install
    Command::new("./configure")
        .arg(&format!("--prefix={}/install", xpdf_dir))
        .env("CC", "/usr/local/bin/afl-cc")
        .env("CFLAGS", format!("{afl_path}/afl-compiler-rt.o"))
        .env("LDFLAGS", format!("{afl_path}/afl-compiler-rt.o"))
        .current_dir(xpdf_dir.clone())
        .status()
        .expect("Couldn't configure xpdf to build using afl-cc");

    // make LDFLAGS="-fsanitize=address $AFL_PATH/afl-compiler-rt.o" && make install
    exec(Command::new("make")
        .current_dir(xpdf_dir.clone()))?;
    exec(Command::new("make")
    .arg("install")
        .current_dir(xpdf_dir))?;


    Ok(())
}
//troysargent@Mac xpdf % CFLAGS="-fsanitize=address -fno-omit-frame-pointer" LDFLAGS="-fsanitize=address" CC=/usr/local/bin/afl-cc ./configure --prefix=$(pwd)/install