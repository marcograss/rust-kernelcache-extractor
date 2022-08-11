extern crate clap;
extern crate kcacheext;

use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let matches = App::new("kcache_extract")
        .version(env!("CARGO_PKG_VERSION"))
        .author("marcograss")
        .about("Extract a decrypted iOS 64-bit kernelcache and kpp if present")
        .arg(
            Arg::with_name("input")
                .short('i')
                .long("input")
                .value_name("INPUT")
                .help("Compressed kernelcache")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help("Output file, the decompressed kernelcache")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("kpp")
                .short('k')
                .long("kpp")
                .value_name("KPP")
                .help("Output file for kpp if present")
                .takes_value(true),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();
    let kernelcache_output_filename = matches.value_of("output").unwrap();

    if Path::new(kernelcache_output_filename).exists() {
        println!("file {:?} already exists", kernelcache_output_filename);
        return;
    }

    match kcacheext::extract_from_file(input_filename) {
        Ok(decoded) => {
            let mut kernelcache_output_file = match File::create(kernelcache_output_filename) {
                Err(why) => panic!("couldn't create {}: {}", kernelcache_output_filename, why),
                Ok(file) => file,
            };
            match kernelcache_output_file.write_all(&decoded.kernelcache) {
                Err(why) => panic!(
                    "couldn't write kernelcache to {}: {}",
                    kernelcache_output_filename, why
                ),
                Ok(_) => println!(
                    "successfully wrote kernelcache to {}",
                    kernelcache_output_filename
                ),
            };
            if matches.is_present("kpp") {
                let kpp_output_filename = matches.value_of("kpp").unwrap();
                if Path::new(kpp_output_filename).exists() {
                    println!("file {:?} already exists", kpp_output_filename);
                    return;
                }
                if decoded.kpp_present {
                    let mut kpp_output_file = match File::create(kpp_output_filename) {
                        Err(why) => panic!("couldn't create {}: {}", kpp_output_filename, why),
                        Ok(file) => file,
                    };
                    match kpp_output_file.write_all(&decoded.kpp) {
                        Err(why) => {
                            panic!("couldn't write kpp to {}: {}", kpp_output_filename, why)
                        }
                        Ok(_) => println!("successfully wrote kpp to {}", kpp_output_filename),
                    };
                } else {
                    println!("There is no kpp in the image");
                }
            } else if decoded.kpp_present {
                println!("There is a kpp, if you need it use --kpp");
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
