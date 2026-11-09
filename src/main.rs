// Copyright 2026 Justin Hu
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

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::{IsTerminal, stdin, stdout},
};

use regex::Regex;

use clap::{Parser, ValueEnum};

fn print_list(names: &HashMap<char, String>, materials: &BTreeMap<char, usize>) {
    for (material, count) in materials.iter() {
        let material_string = material.to_string();
        let name = names.get(material).unwrap_or(&material_string);
        println!("{count}x {name}");
    }
}

fn traverse_recipe_tree(
    recipes: &HashMap<char, HashMap<char, usize>>,
    ingredient: char,
) -> BTreeMap<char, usize> {
    match recipes.get(&ingredient) {
        None => BTreeMap::from([(ingredient, 1)]),
        Some(recipe) => {
            let mut output = BTreeMap::<char, usize>::new();
            for (&ingredient, &count) in recipe {
                for (base_ingredient, base_count) in traverse_recipe_tree(recipes, ingredient) {
                    *output.entry(base_ingredient).or_default() += base_count * count;
                }
            }
            output
        }
    }
}

fn main() {
    #[derive(Clone, Copy, ValueEnum)]
    #[value(rename_all = "kebab-case")]
    enum DisplayMode {
        Both,
        Direct,
        Base,
    }

    #[derive(Parser)]
    struct Args {
        #[arg(long, value_enum, default_value_t = DisplayMode::Both)]
        display: DisplayMode,
    }

    let args = Args::parse();

    if stdin().is_terminal() && stdout().is_terminal() {
        println!("Material Lister version {}", env!("CARGO_PKG_VERSION"));
    }

    // read and parse input
    let mut names = HashMap::<char, String>::new();
    let mut recipes = HashMap::<char, HashMap<char, usize>>::new();
    let mut materials = BTreeMap::<char, usize>::new();

    let naming_regex = Regex::new(r"^([[:alpha:]]) = (.+)$").unwrap();
    let component_regex = Regex::new(r"^([[:alpha:]]) => ((?:[[:digit:]]*[[:alpha:]])+)$").unwrap();
    let material_regex = Regex::new(r"^(?:[[:digit:]]*[[:alpha:]])+$").unwrap();
    let part_regex = Regex::new(r"([[:digit:]]*)([[:alpha:]])").unwrap();

    for line in stdin().lines() {
        use std::collections::hash_map::Entry;

        let line = match line {
            Ok(line) => line,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };

        if let Some(found) = naming_regex.captures(&line) {
            let named = found.get(1).unwrap().as_str().chars().next().unwrap();
            let name = found.get(2).unwrap().as_str().to_owned();

            match names.entry(named) {
                Entry::Occupied(entry) => {
                    eprintln!(
                        "Duplicate name: {named} has names {name} and {}",
                        entry.get()
                    );
                    return;
                }
                Entry::Vacant(entry) => {
                    entry.insert(name);
                }
            }
        } else if let Some(found) = component_regex.captures(&line) {
            let crafted = found.get(1).unwrap().as_str().chars().next().unwrap();
            let ingredients_counts = part_regex
                .captures_iter(found.get(2).unwrap().as_str())
                .map(|part| {
                    let ingredient = part.get(2).unwrap().as_str().chars().next().unwrap();
                    let count = part.get(1).unwrap().as_str();
                    let count = if count.is_empty() {
                        1
                    } else {
                        count.parse::<usize>().unwrap()
                    };
                    (ingredient, count)
                });
            let mut ingredients = HashMap::<char, usize>::new();
            for (ingredient, count) in ingredients_counts {
                *ingredients.entry(ingredient).or_default() += count;
            }

            match recipes.entry(crafted) {
                Entry::Occupied(_) => {
                    eprintln!("Duplicate crafting recipe: {crafted}");
                    return;
                }
                Entry::Vacant(entry) => {
                    entry.insert(ingredients);
                }
            }
        } else if material_regex.is_match(&line) {
            for part in part_regex.captures_iter(&line) {
                let material = part.get(2).unwrap().as_str().chars().next().unwrap();
                let count = part.get(1).unwrap().as_str();
                let count = if count.is_empty() {
                    1
                } else {
                    count.parse::<usize>().unwrap()
                };
                *materials.entry(material).or_default() += count;
            }
        } else if line.trim().is_empty() || line.trim().starts_with("#") {
            continue;
        } else {
            eprintln!("Invalid input: {line}");
            return;
        }
    }

    // check components for cycles
    for (&output, _) in recipes.iter() {
        let mut todo = vec![(output, HashSet::<char>::new())];
        while let Some((current, mut path)) = todo.pop() {
            if !path.insert(current) {
                eprintln!(
                    "Recipe loop detected: while considering {output}, found {current} was an ingredient of itself"
                );
                return;
            }

            if let Some(recipe) = recipes.get(&current) {
                todo.extend(recipe.keys().map(|ingredient| (*ingredient, path.clone())))
            }
        }
    }

    // calculate materials
    if matches!(args.display, DisplayMode::Base) {
        print_list(&names, &materials);
    } else {
        let mut base = BTreeMap::<char, usize>::new();
        for (&material, &count) in materials.iter() {
            for (base_material, base_count) in traverse_recipe_tree(&recipes, material) {
                *base.entry(base_material).or_default() += base_count * count;
            }
        }

        match args.display {
            DisplayMode::Both => {
                println!("Direct:");
                print_list(&names, &materials);
                println!("Base:");
                print_list(&names, &base);
            }
            DisplayMode::Direct => {
                print_list(&names, &base);
            }
            DisplayMode::Base => unreachable!(),
        }
    }
}
