extern crate byteorder;
extern crate bytesize;
extern crate packed_struct;
#[macro_use]
extern crate packed_struct_codegen;

use bytesize::ByteSize;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};

use packed_struct::prelude::*;
use std::mem;

mod lzss;

// Thanks Jonathan
#[derive(PackedStruct)]
#[packed_struct(endian = "msb")]
pub struct CompressionHeader {
    compzlss_str: u64,
    unknown: u32,
    uncompressed_size: u32,
    compressed_size: u32,
    unknown1: u32,
}

pub fn extract_from_file(input_file: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(input_file)?;
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;
    extract_from_buf(&mut buffer)
}

fn find_subsequence<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where
    for<'a> &'a [T]: PartialEq,
{
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub fn extract_from_buf(input_buf: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
    if let Some(lzss_location) = find_subsequence(input_buf, b"complzss") {
        let lzss_header_size = mem::size_of::<CompressionHeader>();
        if lzss_header_size != 24 {
            panic!(
                "CompressionHeader size is wrong! BUG {:?}",
                lzss_header_size
            );
        }
        if (lzss_location + lzss_header_size) > input_buf.len() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Image too small, no complzss header space",
            ));
        }
        let mut header_vec: [u8; 24] = [0; 24];
        header_vec.clone_from_slice(&input_buf[lzss_location..lzss_location + 24]);
        let comp_header = CompressionHeader::unpack(&header_vec).unwrap();
        println!(
            "magic {:#x?} compressed size is: {:?} - uncompressed size is: {:?}",
            comp_header.compzlss_str,
            ByteSize(comp_header.compressed_size as u64),
            ByteSize(comp_header.uncompressed_size as u64)
        );

        if let Some(macho_loc) = find_subsequence(input_buf, b"\xCF\xFA\xED\xFE") {
            println!("Mach-O is at {:?}", macho_loc);
            let mut decoded_buffer: Vec<u8> =
                Vec::with_capacity(comp_header.uncompressed_size as usize);
            let res_deco = lzss::lzss_decode_block_content(
                &mut &input_buf[(macho_loc - 1)..],
                comp_header.compressed_size as u64,
                &mut decoded_buffer,
            );
            match res_deco {
                Ok(decompressed_bytes_res) => {
                    if decompressed_bytes_res != comp_header.uncompressed_size as u64 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "The uncompressed size doesn't match the one in the header",
                        ));
                    }
                    return Ok(decoded_buffer);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Can't find mach-o header",
            ));
        }
    } else {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Can't find complzss magic",
        ));
    }
}
