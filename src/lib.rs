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

#[repr(u16)]
/// The pixel format of a RIM image
pub enum RimFormat
{
    /// 16-bit RGB, with 1 ignored, 5 red, 5 green, and 5 blue bits.
    RGB555 = 0,
    /// 16-bit RGB, with 5 red, 6 green, and 5 blue bits.
    RGB565 = 1,
}

/// The FOURCC signature of a RIM file, 'RIMF'
pub const RIM_SIGNATURE : u32 = 0x464d4952;

/// The raw headers for a Rim Image file, excluding the 'RIMF' signature
pub struct RimHeader
{
    /// The file version, always 0
    pub ver : u32,
    /// The width of the image, in pixels
    pub width : u16,
    /// The height of the image, in pixels
    pub height : u16,
    /// The pitch of the image, in bytes
    pub pitch : u16,
    /// The pixel format of the image. Always 16-bit.
    pub fmt : RimFormat,
}

impl RimHeader {
    /// Read a RimHeader from a stream. Image data immediately follows.
    pub fn from_stream(reader: &mut impl Read) -> std::io::Result<RimHeader> {
        let rim_sig = reader.read_u32::<LittleEndian>()?;

        // 'RIMF'
        if rim_sig != RIM_SIGNATURE {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid RIM signature"));
        }

        let rim_ver = reader.read_u32::<LittleEndian>()?;
        let rim_width = reader.read_u16::<LittleEndian>()?;
        let rim_height = reader.read_u16::<LittleEndian>()?;
        let rim_pitch = reader.read_u16::<LittleEndian>()?;
        let rim_fmt = reader.read_u16::<LittleEndian>()?;

        let fmt = match rim_fmt {
            0 => RimFormat::RGB555,
            1 => RimFormat::RGB565,
            _ => { return Err(std::io::Error::new(io::ErrorKind::Other, "Unknown RIM format")); }
        };

        Ok(RimHeader {
            ver: rim_ver,
            width: rim_width,
            height: rim_height,
            pitch: rim_pitch,
            fmt,
        })
    }

    /// Reads the raw 16-bit pixel data from a stream, compensating for pitch.
    ///
    /// # Arguments
    ///
    /// - reader: The contents of the file. This must be seeked at the start of the image data
    ///    (16 bytes from the start of the file), and must support the Seek trait, as we may seek
    ///    if the image pitch is not equivalent to the width.
    ///
    /// # Return Value
    ///
    /// On success, returns a vector of u16s, each of which represents a pixel in the image's
    /// native format (either RGB555, or RGB565). Use RimHeader::fmt to query the correct format.
    ///
    /// # See Also
    ///
    /// The RimHeader::read_rgba_bytes() function is similar, but returns a byte stream in RGBA
    /// format, converting automatically from the image's native format.
    ///
    /// On failure, returns an io::Error.
    pub fn read_contiguous(&self, reader : &mut (impl Read + Seek)) -> std::io::Result<Vec<u16>> {
        let num_pixels = (self.width * self.height) as usize;
        let mut data = Vec::<u16>::with_capacity(num_pixels);
        for _line_num in 0..self.height {
            for _px in 0..self.width {
                data.push(reader.read_u16::<LittleEndian>()?);
            }

            if self.width * 2 != self.pitch {
                reader.seek(io::SeekFrom::Current((self.pitch - self.width * 2) as i64))?;
            }
        }

        Ok(data)
    }

    /// Reads the image data for a RIM image, converting it to 32-bit RGBA data.
    ///
    /// # Arguments
    ///
    /// - reader: The contents of the file. This must be seeked at the start of the image data
    ///    (16 bytes from the start of the file), and must support the Seek trait, as we may seek
    ///    if the image pitch is not equivalent to the width.
    ///
    /// # Return Value
    ///
    /// On success, returns a vector of width*height*4 u8 bytes, which contain the image data
    /// converted to 32-bit RGBA format.
    ///
    /// # See Also
    ///
    /// The RimHeader::read_continuous() function is similar, but returns the data in its native
    /// 16-bit format, but with rows of pixel data contiguous (i.e., as if pitch == width*2).
    ///
    /// On failure, returns an io::Error.
    pub fn read_rgba_bytes(&self, reader : &mut (impl Read + Seek)) -> std::io::Result<Vec<u8>> {
        let num_pixels = (self.width * self.height) as usize;
        let mut data = Vec::<u8>::with_capacity(num_pixels * 4);
        for _line_num in 0..self.height {
            for _px in 0..self.width {
                match self.fmt {
                    RimFormat::RGB555 => {
                        // 555
                        let px555 = reader.read_u16::<LittleEndian>().unwrap();
                        let red = (px555 >> 10) & 31;
                        let green = (px555 >> 5) & 31;
                        let blue = (px555 >> 0) & 31;
                        data.push((red << 3) as u8);
                        data.push((green << 3) as u8);
                        data.push((blue << 3) as u8);
                        data.push(255);
                    },
                    RimFormat::RGB565 => {
                        // 565
                        let px565 = reader.read_u16::<LittleEndian>().unwrap();
                        let red = (px565 >> 10) & 31;
                        let green = (px565 >> 5) & 63;
                        let blue = (px565 >> 0) & 31;
                        data.push((red << 3) as u8);
                        data.push((green << 2) as u8);
                        data.push((blue << 3) as u8);
                        data.push(255);
                    },
                }
            }

            if self.width * 2 != self.pitch {
                reader.seek(io::SeekFrom::Current((self.pitch - self.width * 2) as i64))?;
            }
        }
        Ok(data)
    }
}

/// Represents a single entry in a ZFS file.
#[derive(Clone)]
pub struct ZfsEntry
{
    /// The filename of the entry.
    pub name : String,
    /// The offset, in bytes, of the file data from the start of the ZFS file.
    pub offset : usize,
    /// The size, in bytes, of the file.
    pub size : usize,
    /// The timestamp of the file.
    pub timestamp : std::time::SystemTime,
    /// Flags stored in the entry header
    pub flags : u32,
}

impl ZfsEntry
{
    /// Read the contents of an entry.
    ///
    /// # Arguments
    ///
    /// - reader: A reader for the whole ZFS file. This needs to implement Seek, as
    ///    get_data() will seek to the start of the entry's data before reading.
    pub fn get_data(&self, reader : &mut (impl Read + Seek)) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; self.size];
        reader.seek(io::SeekFrom::Start(self.offset as u64))?;
        reader.read(&mut buffer)?;
        Ok(buffer)
    }
    
    pub fn extract_file(&self, reader : &mut (impl Read + Seek)) -> io::Result<()> {
        let data = self.get_data(reader)?;
        println!("Extracting file \"{}\"…", self.name);
        let mut out_file = File::create(&self.name)?;
        out_file.write_all(&data)?;
        #[cfg(feature = "set-timestamps")]
        out_file.set_modified(self.timestamp)?;
        Ok(())
    }
    
    pub fn extract_rim_image(&self, reader : &mut (impl Read + Seek)) -> io::Result<()> {
        reader.seek(io::SeekFrom::Start(self.offset as u64))?;
        

        let rim_image = RimHeader::from_stream(reader)?;

        
        let out_file = File::create(format!("{}.png", &self.name))?;
        {
            let mut png_encoder = png::Encoder::new(&out_file, rim_image.width as u32, rim_image.height as u32);
            // Note: Newer versions of the 'png' library call this 'Rgba'.
            png_encoder.set_color(png::ColorType::RGBA);
            png_encoder.set_depth(png::BitDepth::Eight);
            
            let mut px_writer = png_encoder.write_header()?;
            px_writer.write_image_data(&rim_image.read_rgba_bytes(reader)?)?;
        }
        #[cfg(feature = "set-timestamps")]
        out_file.set_modified(self.timestamp)?;
        Ok(())
    }
}

/// The FOURCC signature of a ZFS file, 'ZFS3
pub const ZFS_SIGNATURE : u32 = 0x3353465a;


/// Represents the headers of a ZFS file.
///
/// Unlike ZfsFile, this does not maintain a reference to the data, or the
/// underlying file. Any functions which access the data will need to provide
/// a reader.
pub struct ZfsHeaders
{
    pub version : u32,
    pub max_filename_len : u32,
    pub files : Vec::<ZfsEntry>,
}

impl ZfsHeaders {
    pub fn from_stream(reader : &mut (impl Read + Seek)) -> io::Result<ZfsHeaders> {
        let sig = reader.read_u32::<LittleEndian>()?;
        // 'ZFS3'
        if sig != ZFS_SIGNATURE {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid ZFS signature"));
        }
        let version = reader.read_u32::<LittleEndian>()?;
        let max_filename_len = reader.read_u32::<LittleEndian>()?;
        let files_per_table = reader.read_u32::<LittleEndian>()?;
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
            let flags = reader.read_u32::<LittleEndian>()?;

            files.push(ZfsEntry{
                name : file_name.to_string(),
                offset : data_offset as usize,
                size : data_size as usize,
                timestamp: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64),
                flags
            });

            if (i % files_per_table) == (files_per_table - 1) {
                reader.seek(io::SeekFrom::Start(next_table_offset as u64))?;
                next_table_offset = reader.read_u32::<LittleEndian>()?;
            }
        }

        Ok(ZfsHeaders {
            version,
            max_filename_len,
            files
        })
    }

    pub fn extract_all(self, reader : &mut (impl Read + Seek)) -> io::Result<()> {
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

