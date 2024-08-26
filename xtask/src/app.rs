use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Context;

pub fn gen_app_asm(bins: Vec<String>, dest: &str) -> anyhow::Result<()> {
    let file = File::create(dest).context("create file failed")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "\t.section .data")?;
    writeln!(writer, "\t.globl app_data")?;
    writeln!(writer, "app_data:")?;
    writeln!(writer, "\t.quad {}", bins.len())?;

    for i in 0..bins.len() {
        writeln!(writer, "\t.quad app_{}_start", i)?;
        writeln!(writer, "\t.quad app_{}_end", i)?;
        writeln!(writer, "\t.quad app_{}_name", i)?;
    }

    for (i, bin) in bins.iter().enumerate() {
        let name = Path::new(bin).file_name().unwrap().to_str().unwrap();
        writeln!(writer, "app_{}_start:", i)?;
        writeln!(writer, "\t.incbin \"{}\"", bin)?;
        writeln!(writer, "app_{}_end:", i)?;
        writeln!(writer, "app_{}_name:", i)?;
        writeln!(writer, "\t.string \"{}\"", name)?;
    }

    writer.flush().context("flush failed")?;

    Ok(())
}
