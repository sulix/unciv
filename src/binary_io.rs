/*
 * unciv: An extractor for Civilization: Call to Power's zfs files.
 *
 * Copyright (c) 2025, David Gow <david@davidgow.net>
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

#![allow(dead_code)]

pub fn read_byte(reader: &mut dyn std::io::Read) -> std::io::Result<u8> {
    let mut out_byte: u8 = 0;
    reader.read_exact(std::slice::from_mut(&mut out_byte))?;
    return Ok(out_byte);
}

pub fn read_le16(reader: &mut dyn std::io::Read) -> std::io::Result<u16> {
    let mut raw_bytes = [0 as u8; 2];
    reader.read_exact(&mut raw_bytes)?;
    return Ok(u16::from_le_bytes(raw_bytes));
}

pub fn read_le32(reader: &mut dyn std::io::Read) -> std::io::Result<u32> {
    let mut raw_bytes = [0 as u8; 4];
    reader.read_exact(&mut raw_bytes)?;
    return Ok(u32::from_le_bytes(raw_bytes));
}

pub fn read_be16(reader: &mut dyn std::io::Read) -> std::io::Result<u16> {
    let mut raw_bytes = [0 as u8; 2];
    reader.read_exact(&mut raw_bytes)?;
    return Ok(u16::from_be_bytes(raw_bytes));
}

pub fn read_be32(reader: &mut dyn std::io::Read) -> std::io::Result<u32> {
    let mut raw_bytes = [0 as u8; 4];
    reader.read_exact(&mut raw_bytes)?;
    return Ok(u32::from_be_bytes(raw_bytes));
}

pub fn write_byte(out_byte: u8, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writer.write_all(std::slice::from_ref(&out_byte))
}

pub fn write_be16(out_val: u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = out_val.to_be_bytes();
    writer.write_all(&raw_bytes)
}

pub fn write_be32(out_val: u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = out_val.to_be_bytes();
    writer.write_all(&raw_bytes)
}

pub fn write_le16(out_val: u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = out_val.to_le_bytes();
    writer.write_all(&raw_bytes)
}

pub fn write_le32(out_val: u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = out_val.to_le_bytes();
    writer.write_all(&raw_bytes)
}
