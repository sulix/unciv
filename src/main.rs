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

extern crate png;
extern crate unciv;
use std::fs::File;


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

    let zfs_file = unciv::ZfsHeaders::from_stream(&mut file).unwrap();

    zfs_file.extract_all(&mut file).unwrap();
}
