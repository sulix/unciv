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

#![cfg_attr(feature = "set-timestamps", feature(file_set_times))]

extern crate byteorder;
extern crate png;
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::Seek;
use std::io;
use std::io::Read;
use std::io::Write;

fn convert_555_888<W: std::io::Write>(reader : &mut impl Read, writer: &mut png::StreamWriter<W>) {
    let px555 = reader.read_u16::<LittleEndian>().unwrap();
    let red = (px555 >> 10) & 31;
    let green = (px555 >> 5) & 31;
    let blue = (px555 >> 0) & 31;
    let rgb = [(red << 3) as u8, (green << 3) as u8, (blue << 3) as u8, 255u8];
    writer.write(&rgb).unwrap();
}

fn convert_565_888<W: std::io::Write>(reader : &mut impl Read, writer: &mut png::StreamWriter<W>) {
    let px555 = reader.read_u16::<LittleEndian>().unwrap();
    let red = (px555 >> 11) & 31;
    let green = (px555 >> 5) & 63;
    let blue = (px555 >> 0) & 31;
    let rgb = [(red << 3) as u8, (green << 2) as u8, (blue << 3) as u8, 255 as u8];
    writer.write(&rgb).unwrap();
}

struct ZfsEntry
{
    name : String,
    offset : usize,
    size : usize,
    timestamp : std::time::SystemTime,
}

impl ZfsEntry
{
    fn get_data(&self, reader : &mut (impl Read + Seek)) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; self.size];
        reader.seek(io::SeekFrom::Start(self.offset as u64))?;
        reader.read(&mut buffer)?;
        Ok(buffer)
    }
    
    fn extract_file(&self, reader : &mut (impl Read + Seek)) -> io::Result<()> {
        let data = self.get_data(reader)?;
        println!("Extracting file \"{}\"…", self.name);
        let mut out_file = File::create(&self.name)?;
        out_file.write_all(&data)?;
        #[cfg(feature = "set-timestamps")]
        out_file.set_modified(self.timestamp)?;
        Ok(())
    }
    
    fn extract_rim_image(&self, reader : &mut (impl Read + Seek)) -> io::Result<()> {
        reader.seek(io::SeekFrom::Start(self.offset as u64))?;
        
        let rim_sig = reader.read_u32::<LittleEndian>()?;
        
        // 'RIMF'
        if rim_sig != 0x464d4952 {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid RIM signature"));
        }
        
        let rim_ver = reader.read_u32::<LittleEndian>()?;
        let rim_width = reader.read_u16::<LittleEndian>()?;
        let rim_height = reader.read_u16::<LittleEndian>()?;
        let rim_pitch = reader.read_u16::<LittleEndian>()?;
        let rim_fmt = reader.read_u16::<LittleEndian>()?;
        
        println!("Converting RIM v{} ({}) image \"{}\" ({}×{})…", rim_ver, if rim_fmt == 0 { "RGB555" } else { "RGB565" }, self.name, rim_width, rim_height);
        
        // Note: Apparently this doesn't need to be mutable? Can't think of why, though: it has state (e.g. offset) we're mutating.
        let mut out_file = File::create(format!("{}.png", &self.name))?;
        {
            let mut png_encoder = png::Encoder::new(&out_file, rim_width as u32, rim_height as u32);
            // Note: Newer versions of the 'png' library call this 'Rgba'.
            png_encoder.set_color(png::ColorType::RGBA);
            png_encoder.set_depth(png::BitDepth::Eight);
            
            let mut px_writer = png_encoder.write_header()?;
            // Note: Newer versions of the 'png' library have this return a result, so needs a '?'.
            let mut stream_writer = px_writer.stream_writer();
            for _line_num in 0..rim_height {
                for _px in 0..rim_width {
                    if rim_fmt == 0 {
                        convert_555_888(reader, &mut stream_writer);
                    } else if rim_fmt == 1 {
                        convert_565_888(reader, &mut stream_writer);
                    }

                }
                if rim_width * 2 < rim_pitch {
                    reader.seek(io::SeekFrom::Current((rim_pitch - rim_width * 2) as i64))?;
                }
            }
        }
        #[cfg(feature = "set-timestamps")]
        out_file.set_modified(self.timestamp)?;
        Ok(())
    }
}

struct ZfsFile
{
    _version : u32,
    _max_filename_len : u32,
    files : Vec::<ZfsEntry>,
}

impl ZfsFile
{
    fn from_stream(reader : &mut (impl Read + Seek)) -> io::Result<ZfsFile> {
        let sig = reader.read_u32::<LittleEndian>()?;
        // 'ZFS3'
        if sig != 0x3353465a {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid ZFS signature"));
        }
        let version = reader.read_u32::<LittleEndian>()?;
        let max_filename_len = reader.read_u32::<LittleEndian>()?;
        let _unk1 = reader.read_u32::<LittleEndian>()?;
        let num_files = reader.read_u32::<LittleEndian>()?;
        let _unk2 = reader.read_u32::<LittleEndian>()?;
        let filetable_offset = reader.read_u32::<LittleEndian>()?;


        let mut files = Vec::<ZfsEntry>::new();

        reader.seek(io::SeekFrom::Start(filetable_offset as u64))?;

        let mut next_table_offset = reader.read_u32::<LittleEndian>()?;
        for i in 0..num_files {
            let mut raw_name = vec![0; max_filename_len as usize];
            reader.read_exact(&mut raw_name)?;

            if raw_name[0] == 0 {
                break;
            }

            let file_name = String::from_utf8_lossy(&raw_name);
            let file_name = file_name.trim_matches('\0');
            let data_offset = reader.read_u32::<LittleEndian>()?;
            let _unk3 = reader.read_u32::<LittleEndian>()?;
            let data_size = reader.read_u32::<LittleEndian>()?;
            let timestamp = reader.read_u32::<LittleEndian>()?;
            let _flags = reader.read_u32::<LittleEndian>()?;

            files.push(ZfsEntry{
                name : file_name.to_string(),
                offset : data_offset as usize,
                size : data_size as usize,
                timestamp: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64),
            });

            if (i % _unk1) == (_unk1 - 1) {
                reader.seek(io::SeekFrom::Start(next_table_offset as u64))?;
                next_table_offset = reader.read_u32::<LittleEndian>()?;
            }
        }

        Ok(ZfsFile {
            _version : version,
            _max_filename_len : max_filename_len,
            files
        })
    }
    
    fn extract_all(self, reader : &mut (impl Read + Seek)) -> io::Result<()> {
        for i in self.files {
            if i.name.ends_with(".rim") {
                i.extract_rim_image(reader)?;
            } else {
                i.extract_file(reader)?;
            }
        }
        Ok(())
    }
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

    let zfs_file = ZfsFile::from_stream(&mut file).unwrap();
    
    zfs_file.extract_all(&mut file).unwrap();
}
