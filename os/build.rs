use std::fs::{read_dir, File};
use std::io::{Result, Write};

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().unwrap();
}

static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            let position = name_with_ext.find('.');
            match position {
                Some(pos) => {
                    name_with_ext.drain(pos..name_with_ext.len());
                    Some(name_with_ext)
                }
                None => { /* Just a dictionary. */ None }
            }
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
    .align 3
    .section .rodata
    .global _num_app
_num_app:
    .quad {}
    .align 3
    .section .rodata
    .global _app_meta
_app_meta:"#,
        apps.len()
    )?;

    for (idx, app) in apps.iter().enumerate() {
        writeln!(f, r#"    .quad app_{}_start"#, idx)?;
        writeln!(f, r#"    .quad app_{}_end - app_{}_start"#, idx, idx)?;
        writeln!(f, r#"    .quad .L_name_{}"#, idx)?;
        writeln!(f, r#"    .quad {}"#, app.len())?;
    }

    writeln!(f,"")?;

    for (idx, app) in apps.iter().enumerate() {
        writeln!(f, r#".L_name_{}:"#, idx)?;
        writeln!(f, r#"    .string "{}""#, app)?;
    }

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .rodata
    .global app_{0}_start
    .global app_{0}_end
    .align 3
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}
