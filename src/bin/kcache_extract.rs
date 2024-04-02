use anyhow::anyhow;
use clap::{Arg, ArgAction, Command};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let matches = Command::new("kcache_extract")
        .version(env!("CARGO_PKG_VERSION"))
        .author("marcograss")
        .about("Extract a decrypted iOS 64-bit kernelcache and kpp if present")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("INPUT")
                .help("Compressed kernelcache")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help("Output file, the decompressed kernelcache")
                .required(true),
        )
        .arg(
            Arg::new("kpp")
                .short('k')
                .long("kpp")
                .value_name("KPP")
                .help("Output file for kpp if present"),
        )
        .arg(
            Arg::new("count-symbols")
                .short('s')
                .long("syms")
                .action(ArgAction::SetTrue)
                .help("Count the symbols in the kernelcache"),
        )
        .get_matches();

    let input_filename = matches.get_one::<String>("input").unwrap();
    let kernelcache_output_filename = matches.get_one::<String>("output").unwrap();

    if Path::new(kernelcache_output_filename).exists() {
        return Err(anyhow!(
            "file {kernelcache_output_filename:?} already exists"
        ));
    }

    let decoded = kcacheext::extract_from_file(input_filename)?;
    let mut kernelcache_output_file = File::create(kernelcache_output_filename)?;
    kernelcache_output_file.write_all(&decoded.kernelcache)?;
    if matches.contains_id("kpp") {
        let kpp_output_filename = matches.get_one::<String>("kpp").unwrap();
        if Path::new(kpp_output_filename).exists() {
            return Err(anyhow!("file {kpp_output_filename:?} already exists"));
        }
        if decoded.kpp_present {
            let mut kpp_output_file = File::create(kpp_output_filename)?;
            kpp_output_file.write_all(&decoded.kpp)?;
        } else {
            println!("There is no kpp in the image");
        }
    } else if decoded.kpp_present {
        println!("There is a kpp, if you need it use --kpp");
    }
    if matches.get_flag("count-symbols") {
        let sym_nums = kcacheext::count_symbols(&decoded.kernelcache)?;
        println!("kernelcache has {sym_nums} symbols");
    }
    Ok(())
}
