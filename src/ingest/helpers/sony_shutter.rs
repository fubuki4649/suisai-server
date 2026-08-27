//! Pure-Rust Sony MakerNotes Tag 0x9050 / 0x2010 decipherer and shutter count extractor.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Decipher lookup table: maps encrypted byte → plaintext byte (b^3 mod 249).
static SONY_DECIPHER_TABLE: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = i as u8;
        i += 1;
    }

    // 246 encrypted bytes corresponding to plaintext 0x02..0xF7
    let enc: [u8; 246] = [
        0x08, 0x1b, 0x40, 0x7d, 0xd8, 0x5e, 0x0e, 0xe7, 0x04, 0x56, 0xea, 0xcd, 0x05, 0x8a, 0x70,
        0xb6, 0x69, 0x88, 0x20, 0x30, 0xbe, 0xd7, 0x81, 0xbb, 0x92, 0x0c, 0x28, 0xec, 0x6c, 0xa0,
        0x95, 0x51, 0xd3, 0x2f, 0x5d, 0x6a, 0x5c, 0x39, 0x07, 0xc5, 0x87, 0x4c, 0x1a, 0xf0, 0xe2,
        0xef, 0x24, 0x79, 0x02, 0xb7, 0xac, 0xe0, 0x60, 0x2b, 0x47, 0xba, 0x91, 0xcb, 0x75, 0x8e,
        0x23, 0x33, 0xc4, 0xe3, 0x96, 0xdc, 0xc2, 0x4e, 0x7f, 0x62, 0xf6, 0x4f, 0x65, 0x45, 0xee,
        0x74, 0xcf, 0x13, 0x38, 0x4b, 0x52, 0x53, 0x54, 0x5b, 0x6e, 0x93, 0xd0, 0x32, 0xb1, 0x61,
        0x41, 0x57, 0xa9, 0x44, 0x27, 0x58, 0xdd, 0xc3, 0x10, 0xbc, 0xdb, 0x73, 0x83, 0x18, 0x31,
        0xd4, 0x15, 0xe5, 0x5f, 0x7b, 0x46, 0xbf, 0xf3, 0xe8, 0xa4, 0x2d, 0x82, 0xb0, 0xbd, 0xaf,
        0x8c, 0x5a, 0x1f, 0xda, 0x9f, 0x6d, 0x4a, 0x3c, 0x49, 0x77, 0xcc, 0x55, 0x11, 0x06, 0x3a,
        0xb3, 0x7e, 0x9a, 0x14, 0xe4, 0x25, 0xc8, 0xe1, 0x76, 0x86, 0x1e, 0x3d, 0xe9, 0x36, 0x1c,
        0xa1, 0xd2, 0xb5, 0x50, 0xa2, 0xb8, 0x98, 0x48, 0xc7, 0x29, 0x66, 0x8b, 0x9e, 0xa5, 0xa6,
        0xa7, 0xae, 0xc1, 0xe6, 0x2a, 0x85, 0x0b, 0xb4, 0x94, 0xaa, 0x03, 0x97, 0x7a, 0xab, 0x37,
        0x1d, 0x63, 0x16, 0x35, 0xc6, 0xd6, 0x6b, 0x84, 0x2e, 0x68, 0x3f, 0xb2, 0xce, 0x99, 0x19,
        0x4d, 0x42, 0xf7, 0x80, 0xd5, 0x0a, 0x17, 0x09, 0xdf, 0xad, 0x72, 0x34, 0xf2, 0xc0, 0x9d,
        0x8f, 0x9c, 0xca, 0x26, 0xa8, 0x64, 0x59, 0x8d, 0x0d, 0xd1, 0xed, 0x67, 0x3e, 0x78, 0x22,
        0x3b, 0xc9, 0xd9, 0x71, 0x90, 0x43, 0x89, 0x6f, 0xf4, 0x2c, 0x0f, 0xa3, 0xf5, 0x12, 0xeb,
        0x9b, 0x21, 0x7c, 0xb9, 0xde, 0xf1,
    ];

    let mut j = 0;
    while j < 246 {
        table[enc[j] as usize] = (j + 2) as u8;
        j += 1;
    }
    table
};

#[inline]
fn u16_at(data: &[u8], off: usize, is_le: bool) -> Option<u16> {
    let b = data.get(off..off + 2)?;
    Some(if is_le { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) })
}

#[inline]
fn u32_at(data: &[u8], off: usize, is_le: bool) -> Option<u32> {
    let b = data.get(off..off + 4)?;
    Some(if is_le { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) } else { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) })
}

#[inline]
fn read_actuation(block: &[u8], off: usize) -> Option<i64> {
    let val = (u32_at(block, off, true)? & 0x00FFFFFF) as i64;
    if (1..50_000_000).contains(&val) { Some(val) } else { None }
}

/// Iterates over TIFF IFD entries as `(tag_id, count, value_or_offset)`.
fn ifd_entries(data: &[u8], ifd_offset: usize, is_le: bool) -> impl Iterator<Item = (u16, usize, usize)> + '_ {
    let count = u16_at(data, ifd_offset, is_le).unwrap_or(0) as usize;
    (0..count).filter_map(move |i| {
        let off = ifd_offset + 2 + i * 12;
        Some((u16_at(data, off, is_le)?, u32_at(data, off + 4, is_le)? as usize, u32_at(data, off + 8, is_le)? as usize))
    })
}

/// Locates TIFF base offset in raw ARW/TIFF or JPEG APP1 EXIF data.
fn find_tiff_base(data: &[u8]) -> Option<usize> {
    if matches!(data.get(..4), Some(b"II*\0" | b"MM\0*")) {
        return Some(0);
    }
    if data.starts_with(&[0xFF, 0xD8]) {
        let mut off = 2;
        while off + 4 < data.len() && data[off] == 0xFF {
            let marker = data[off + 1];
            let len = u16::from_be_bytes([data[off + 2], data[off + 3]]) as usize;
            if len < 2 { break; }
            if marker == 0xE1 && data.get(off + 4..off + 10) == Some(b"Exif\0\0") {
                let start = off + 10;
                if matches!(data.get(start..start + 4), Some(b"II*\0" | b"MM\0*")) {
                    return Some(start);
                }
            }
            if matches!(marker, 0xDA | 0xD9) { break; }
            off += 2 + len;
        }
    }
    None
}

/// Extracts decrypted Tag 0x9050 / 0x2010 payload from image bytes.
fn extract_deciphered_tag9050(data: &[u8]) -> Option<Vec<u8>> {
    let tiff_base = find_tiff_base(data)?;
    let is_le = data.get(tiff_base..tiff_base + 2)? == b"II";
    let ifd0 = tiff_base.checked_add(u32_at(data, tiff_base + 4, is_le)? as usize)?;

    let (mn_rel, mn_len) = ifd_entries(data, ifd0, is_le)
        .find(|&(tag, ..)| tag == 0x927C)
        .map(|(_, count, val)| (val, count))
        .or_else(|| {
            let exif_rel = ifd_entries(data, ifd0, is_le).find(|&(tag, ..)| tag == 0x8769)?.2;
            let exif_off = tiff_base.checked_add(exif_rel)?;
            ifd_entries(data, exif_off, is_le)
                .find(|&(tag, ..)| tag == 0x927C)
                .map(|(_, count, val)| (val, count))
        })?;

    let mn_abs = tiff_base.checked_add(mn_rel)?;
    let mn_data = data.get(mn_abs..mn_abs.checked_add(mn_len)?)?;

    let ifd_start = if matches!(mn_data.get(..7), Some(b"SONY DS" | b"SONY CA" | b"SONY MO" | b"SONY PI")) { 12 } else { 0 };

    let (val_or_offset, tag_count) = ifd_entries(mn_data, ifd_start, is_le)
        .filter(|&(tag, ..)| tag == 0x9050 || tag == 0x2010)
        .min_by_key(|&(tag, ..)| if tag == 0x9050 { 0 } else { 1 })
        .map(|(_, count, val)| (val, count))?;

    let tag_slice = tiff_base.checked_add(val_or_offset)
        .and_then(|abs| data.get(abs..abs + tag_count))
        .or_else(|| mn_data.get(val_or_offset..val_or_offset + tag_count))?;

    Some(tag_slice.iter().map(|&b| SONY_DECIPHER_TABLE[b as usize]).collect())
}

/// Extracts Sony shutter count from in-memory image bytes.
///
/// Routes closed legacy sets (Gen 1 at 0x0032/0x004C, Gen 2 at 0x003A) while defaulting
/// all current and future models (Tag9050d schema at 0x000A) with zero hardcoding.
pub fn extract_sony_shutter_count_from_bytes(data: &[u8], model: Option<&str>) -> Option<i64> {
    let block = extract_deciphered_tag9050(data)?;

    if let Some(m) = model.map(str::trim) {
        // Gen 1 closed set (2010–2015): NEX, SLT, early A7/A6000
        if m.starts_with("NEX-") || m.starts_with("SLT-") || m.starts_with("ILCE-3")
            || m.starts_with("ILCE-5") || m.starts_with("ILCE-6000")
            || matches!(m, "ILCE-7" | "ILCE-7R" | "ILCE-7S" | "ILCE-7M2")
            || m.starts_with("DSC-RX1")
        {
            return read_actuation(&block, 0x0032).or_else(|| read_actuation(&block, 0x004C));
        }
        // Gen 2 closed set (2015–2022): A7 III/IV, A7R II..V, A9, A1, A6100..6600, FX3
        if !matches!(m, "ILCE-1M2" | "ILCE-9M3" | "ILCE-7CM2" | "ILCE-7CR" | "ZV-E10M2")
            && (m.starts_with("ILCE-6") || m.starts_with("ILCE-7") || m.starts_with("ILCE-9")
                || m.starts_with("ILCE-1") || m.starts_with("ILME-FX3") || m.starts_with("ZV-E10")
                || m.starts_with("ILCA-99M2"))
        {
            return read_actuation(&block, 0x003A);
        }
        // Open-ended default for latest and future models (Tag9050d schema: 2023+ and beyond)
        if let Some(sc) = read_actuation(&block, 0x000A) {
            return Some(sc);
        }
    }

    // Auto-detect fallback: Tag9050d (leading zeros) -> candidate offsets
    if block.len() >= 14 && block[..10].iter().all(|&b| b == 0)
        && let Some(sc) = read_actuation(&block, 0x000A)
    {
        return Some(sc);
    }

    const CANDIDATES: &[usize] = &[0x003A, 0x0032, 0x004C, 0x000A, 0x0050, 0x0052, 0x0058, 0x019F, 0x01CB, 0x01CD];
    CANDIDATES.iter().find_map(|&off| read_actuation(&block, off))
}

/// Reads the header portion of an image file and extracts the Sony shutter count.
pub fn extract_sony_shutter_count(path: &Path, model: Option<&str>) -> Option<i64> {
    let mut buffer = vec![0u8; 2 * 1024 * 1024];
    let n = File::open(path).ok()?.read(&mut buffer).ok()?;
    extract_sony_shutter_count_from_bytes(&buffer[..n], model)
}
