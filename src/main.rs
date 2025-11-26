// Copyright 2025 Justin Hu
//
// This file is part of the Material Lister.
//
// The Material Lister is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// The Material Lister is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
// or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
// more details.
//
// You should have received a copy of the GNU General Public License along with
// the Material Lister. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{collections::HashMap, io::stdin};

use regex::Regex;

fn main() {
    println!("Material Lister version {}", env!("CARGO_PKG_VERSION"));

    let mut total = HashMap::<char, u64>::new();

    let validator = Regex::new(r"^(?:\d*\D)+$").expect("static regex");
    let component_matcher = Regex::new(r"(\d*)(\D)").expect("static regex");
    while let Ok(line) = {
        let mut line = String::new();
        stdin().read_line(&mut line).map(|_| line)
    } {
        let line = line.trim();
        if line.is_empty() {
            break;
        }

        if !validator.is_match(line) {
            println!("Could not parse {line}; ignoring");
            continue;
        }

        let parsed = component_matcher.captures_iter(line);
        for component in parsed {
            let count = component.get(1).expect("mandatory capture group").as_str();
            let count = if count.is_empty() {
                1
            } else {
                count.parse::<u64>().expect("validated by regex")
            };
            let material = component
                .get(2)
                .expect("mandatory capture group")
                .as_str()
                .chars()
                .next()
                .expect("validated by regex");
            *total.entry(material).or_default() += count;
        }
    }

    let mut total = total.into_iter().collect::<Vec<_>>();
    total.sort_by_key(|(material, _)| *material);
    println!("Totals:");
    for (material, count) in total {
        println!("{count}x {material}");
    }
}
