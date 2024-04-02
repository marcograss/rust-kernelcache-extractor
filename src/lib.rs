use bytesize::ByteSize;
use std::fs::File;
use std::io::prelude::*;

use anyhow::{anyhow, Result};
use goblin::mach::Mach;
use lzfse::decode_buffer;
use packed_struct::prelude::*;
use std::mem;

mod lzss;

// Thanks Jonathan
#[derive(PackedStruct)]
#[packed_struct(endian = "msb")]
pub struct CompressionHeader {
    pub compzlss_str: u64,
    pub unknown: u32,
    pub uncompressed_size: u32,
    pub compressed_size: u32,
    pub unknown1: u32,
}

pub struct ExtractionOutput {
    pub kernelcache: Vec<u8>,
    pub kpp_present: bool,
    pub kpp: Vec<u8>,
}
/// Extract the kernelcache from a lzss compressed file
///
/// # Errors
/// can error for many reasons but mostly malformed data
pub fn extract_from_file(input_file: &str) -> Result<ExtractionOutput> {
    let mut file = File::open(input_file)?;
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;
    extract_from_buf(&buffer)
}

fn find_subsequence<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where
    for<'a> &'a [T]: PartialEq,
{
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Extract the kernelcache from a lzss compressed buffer
///
/// # Errors
/// can error for many reasons but mostly malformed data
///
/// # Panics
/// can panic only if we forgot to update the `CompressionHeader` size in the code
pub fn extract_from_buf(input_buf: &[u8]) -> Result<ExtractionOutput> {
    let mut result = ExtractionOutput {
        kernelcache: Vec::new(),
        kpp_present: false,
        kpp: Vec::new(),
    };
    if let Some(lzss_location) = find_subsequence(input_buf, b"complzss") {
        println!("the kernelcache is compressed with LZSS");
        let lzss_header_size = mem::size_of::<CompressionHeader>();
        assert!(
            lzss_header_size == 24,
            "CompressionHeader size is wrong! BUG {lzss_header_size:?}"
        );
        if (lzss_location + lzss_header_size) > input_buf.len() {
            return Err(anyhow!("Image too small, no complzss header space",));
        }
        let mut header_vec: [u8; 24] = [0; 24];
        header_vec.clone_from_slice(&input_buf[lzss_location..lzss_location + 24]);
        let comp_header = CompressionHeader::unpack(&header_vec).unwrap();
        println!(
            "magic {:#x?} compressed size is: {:?} - uncompressed size is: {:?}",
            comp_header.compzlss_str,
            ByteSize(u64::from(comp_header.compressed_size)),
            ByteSize(u64::from(comp_header.uncompressed_size))
        );

        if let Some(imageend) = find_subsequence(&input_buf[0x2000..], b"__IMAGEEND") {
            // println!("imageend at {:?}", imageend);
            if imageend > 0x1000 {
                let start_search_kpp = imageend - 0x1000;
                if let Some(macho_loc) =
                    find_subsequence(&input_buf[start_search_kpp..], b"\xCF\xFA\xED\xFE")
                {
                    let kpp_loc = start_search_kpp + macho_loc;
                    println!("kpp Mach-O is at {kpp_loc:?}");
                    result.kpp.extend_from_slice(&input_buf[kpp_loc..]);
                    result.kpp_present = true;
                }
            }
        }

        if let Some(macho_loc) = find_subsequence(input_buf, b"\xCF\xFA\xED\xFE") {
            println!("kernelcache Mach-O is at {macho_loc:?}");
            let mut decoded_buffer: Vec<u8> =
                Vec::with_capacity(comp_header.uncompressed_size as usize);
            let decompressed_bytes_res = lzss::decode_block_content(
                &mut &input_buf[(macho_loc - 1)..],
                u64::from(comp_header.compressed_size),
                &mut decoded_buffer,
            )?;
            if decompressed_bytes_res != u64::from(comp_header.uncompressed_size) {
                return Err(anyhow!(
                    "The uncompressed size doesn't match the one in the header",
                ));
            }
            result.kernelcache = decoded_buffer;
            Ok(result)
        } else {
            Err(anyhow!("Can't find kernelcache mach-o header",))
        }
    } else if let Some(lzfse_location) = find_subsequence(input_buf, b"bvx2") {
        println!("the kernelcache is compressed with LZFSE");
        // println!("offset is {:?}", lzfse_location);

        // TODO get the dimension of uncompressed data dynamically by parsing asn1
        // TODO for now hardcoded to 200 megs which is enough for now
        let mut uncompressed = vec![0; (1024 * 1024 * 200 + 1) as usize];
        let bytes_decoded =
            decode_buffer(&input_buf[lzfse_location..], &mut uncompressed[..]).unwrap();
        result.kernelcache = uncompressed[..bytes_decoded].to_vec();
        Ok(result)
    } else {
        Err(anyhow!("Can't find comlzss magic"))
    }
}

/// Returns how many symbols a kernel cache have
///
/// # Errors
/// can error on parsing and malformed images
pub fn count_symbols(kc_buffer: &[u8]) -> Result<u64> {
    if let Ok(Mach::Binary(parse_res)) = Mach::parse(kc_buffer) {
        if let Some(symbols) = parse_res.symbols {
            return Ok(symbols.into_iter().count().try_into()?);
        }
        return Ok(0);
    }
    Err(anyhow!("cannot parse kernelcache macho to get the symbols"))
}
