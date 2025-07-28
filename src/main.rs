/*
 * unciv: An extractor for Civilization: Call to Power's zfs files.
 *
 * Copyright (c) 2023, David Gow <david@davidgow.net>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR
 * IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

#![cfg_attr(feature = "set-timestamps", allow(stable_features))]
#![cfg_attr(feature = "set-timestamps", feature(file_set_times))]

extern crate png;
extern crate unciv;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

pub fn save_rim_image(entry : &unciv::ZfsEntry, reader : &mut (impl Read + Seek)) -> std::io::Result<()> {
    let rim_image = entry.read_rim_image(reader)?;


    let out_file = File::create(format!("{}.png", &entry.name))?;
    {
        let mut png_encoder = png::Encoder::new(&out_file, rim_image.width as u32, rim_image.height as u32);
        // Note: Newer versions of the 'png' library call this 'Rgba'.
        png_encoder.set_color(png::ColorType::RGBA);
        png_encoder.set_depth(png::BitDepth::Eight);

        let mut px_writer = png_encoder.write_header()?;
        px_writer.write_image_data(&rim_image.to_rgba_bytes())?;
    }
    #[cfg(feature = "set-timestamps")]
    out_file.set_modified(entry.timestamp)?;
    Ok(())
}

fn main() {
    let args : Vec<std::string::String> = std::env::args().collect();

    if args.len() != 2 {
        println!("unciv: An Uncivilized File Extractor for Civilization: Call to Power");
        println!("By David Gow <david@davidgow.net>");
        println!("");
        println!("Usage: unciv <zfs-file>");
        return;
    }

    println!("File: {}", &args[1]);

    let mut file = File::open(&args[1]).unwrap();

    let zfs_file = unciv::ZfsFile::from_stream(&mut file).unwrap();

    for entry in zfs_file.files {
        if entry.name.ends_with(".rim") {
            save_rim_image(&entry, &mut file).unwrap();
        } else {
            let data = entry.read_data(&mut file).unwrap();
            println!("Extracting file \"{}\"…", entry.name);
            let mut out_file = File::create(entry.name).unwrap();
            out_file.write_all(&data).unwrap();
            #[cfg(feature = "set-timestamps")]
            out_file.set_modified(entry.timestamp).unwrap();
        }
    }
}
