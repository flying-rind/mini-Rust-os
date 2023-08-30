use std::fs::{read_dir, File};
use std::io::{Result, Write};

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    insert_app_data().unwrap();
}

static TARGET_PATH: &str = "../user/target/x86_64/debug/";

fn insert_app_data() -> Result<()> {
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    let mut f = File::create("src/link_app.S").unwrap();
    writeln!(
        f,
        ".align 8\n.data\n.global _app_count\n_app_count:\n.quad {}",
        apps.len()
    )?;
    for i in 0..apps.len() {
        writeln!(f, ".quad app_{}_name\n.quad app_{}_start", i, i)?;
    }
    writeln!(f, ".quad app_{}_end", apps.len() - 1)?;
    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
.data
app_{0}_name:
  .string "{1}"
.align 8
app_{0}_start:
  .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}
