extern crate clap;
extern crate kcacheext;

use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let matches = App::new("kcache_extract")
        .version(env!("CARGO_PKG_VERSION"))
        .author("marcograss")
        .about("Extract a decrypted iOS 64-bit kernelcache")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .help("Compressed kernelcache")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Output file, the decompressed kernelcache")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();
    let output_filename = matches.value_of("output").unwrap();
    let mut output_file = match File::create(output_filename) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't create {}: {}", output_filename, why.description()),
        Ok(file) => file,
    };
    match kcacheext::extract_from_file(input_filename) {
        Ok(decoded) => match output_file.write_all(&decoded) {
            Err(why) => panic!(
                "couldn't write to {}: {}",
                output_filename,
                why.description()
            ),
            Ok(_) => println!("successfully wrote kernelcache to {}", output_filename),
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
