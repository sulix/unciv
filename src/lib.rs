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

mod binary_io;
use binary_io::*;
use std::io::Seek;
use std::io;
use std::io::Read;

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

/// An image in RIM format, including any data.
pub struct RimImage
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
    /// The raw pixel data of the image, as width*height u16s.
    pub data : Vec<u16>,
}

impl RimImage {
    /// Read a RimHeader from a stream. Image data immediately follows.
    pub fn from_stream(reader: &mut (impl Read + Seek)) -> std::io::Result<RimImage> {
        let rim_sig = read_le32(reader)?;

        // 'RIMF'
        if rim_sig != RIM_SIGNATURE {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid RIM signature"));
        }

        let rim_ver = read_le32(reader)?;
        let rim_width = read_le16(reader)?;
        let rim_height = read_le16(reader)?;
        let rim_pitch = read_le16(reader)?;
        let rim_fmt = read_le16(reader)?;

        let fmt = match rim_fmt {
            0 => RimFormat::RGB555,
            1 => RimFormat::RGB565,
            _ => { return Err(std::io::Error::new(io::ErrorKind::Other, "Unknown RIM format")); }
        };

        let num_pixels = rim_width as usize * rim_height as usize;
        let mut data = Vec::<u16>::with_capacity(num_pixels);
        for _line_num in 0..rim_height {
            for _px in 0..rim_width {
                data.push(read_le16(reader)?);
            }

            if rim_width * 2 != rim_pitch {
                reader.seek(io::SeekFrom::Current((rim_pitch - rim_width * 2) as i64))?;
            }
        }

        Ok(RimImage {
            ver: rim_ver,
            width: rim_width,
            height: rim_height,
            pitch: rim_pitch,
            fmt,
            data,
        })
    }

    /// Reads the image data for a RIM image, converting it to 32-bit RGBA data.
    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let num_pixels = self.width as usize * self.height as usize;
        let mut data = Vec::<u8>::with_capacity(num_pixels * 4);
        let mut i = 0;
        for _line_num in 0..self.height {
            for _px in 0..self.width {
                match self.fmt {
                    RimFormat::RGB555 => {
                        // 555
                        let px555 = self.data[i];
                        let red = (px555 >> 10) & 31;
                        let green = (px555 >> 5) & 31;
                        let blue = (px555 >> 0) & 31;
                        data.push((red << 3) as u8);
                        data.push((green << 3) as u8);
                        data.push((blue << 3) as u8);
                        data.push(255);
                        i += 1;
                    },
                    RimFormat::RGB565 => {
                        // 565
                        let px565 = self.data[i];
                        let red = (px565 >> 11) & 31;
                        let green = (px565 >> 5) & 63;
                        let blue = (px565 >> 0) & 31;
                        data.push((red << 3) as u8);
                        data.push((green << 2) as u8);
                        data.push((blue << 3) as u8);
                        data.push(255);
                        i += 1;
                    },
                }
            }
        }
        data
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
    pub fn read_data(&self, reader : &mut (impl Read + Seek)) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; self.size];
        reader.seek(io::SeekFrom::Start(self.offset as u64))?;
        reader.read(&mut buffer)?;
        Ok(buffer)
    }
    
    pub fn read_rim_image(&self, reader : &mut (impl Read + Seek)) -> io::Result<RimImage> {
        reader.seek(io::SeekFrom::Start(self.offset as u64))?;
        let rim_image = RimImage::from_stream(reader)?;
        Ok(rim_image)
    }
}

/// The FOURCC signature of a ZFS file, 'ZFS3
pub const ZFS_SIGNATURE : u32 = 0x3353465a;


/// Represents the headers of a ZFS file.
///
/// Unlike ZfsFile, this does not maintain a reference to the data, or the
/// underlying file. Any functions which access the data will need to provide
/// a reader.
pub struct ZfsFile
{
    pub version : u32,
    pub max_filename_len : u32,
    pub files : Vec::<ZfsEntry>,
}

impl ZfsFile {
    pub fn from_stream(reader : &mut (impl Read + Seek)) -> io::Result<ZfsFile> {
        let sig = read_le32(reader)?;
        // 'ZFS3'
        if sig != ZFS_SIGNATURE {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid ZFS signature"));
        }
        let version = read_le32(reader)?;
        let max_filename_len = read_le32(reader)?;
        let files_per_table = read_le32(reader)?;
        let num_files = read_le32(reader)?;
        let _unk2 = read_le32(reader)?;
        let filetable_offset = read_le32(reader)?;


        let mut files = Vec::<ZfsEntry>::new();

        reader.seek(io::SeekFrom::Start(filetable_offset as u64))?;

        let mut next_table_offset = read_le32(reader)?;
        for i in 0..num_files {
            let mut raw_name = vec![0; max_filename_len as usize];
            reader.read_exact(&mut raw_name)?;

            if raw_name[0] == 0 {
                break;
            }

            let file_name = String::from_utf8_lossy(&raw_name);
            let file_name = file_name.trim_matches('\0');
            let data_offset = read_le32(reader)?;
            let _unk3 = read_le32(reader)?;
            let data_size = read_le32(reader)?;
            let timestamp = read_le32(reader)?;
            let flags = read_le32(reader)?;

            files.push(ZfsEntry{
                name : file_name.to_string(),
                offset : data_offset as usize,
                size : data_size as usize,
                timestamp: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64),
                flags
            });

            if (i % files_per_table) == (files_per_table - 1) {
                reader.seek(io::SeekFrom::Start(next_table_offset as u64))?;
                next_table_offset = read_le32(reader)?;
            }
        }

        Ok(ZfsFile {
            version,
            max_filename_len,
            files
        })
    }


}

