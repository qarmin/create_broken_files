use std::cmp::min;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use rand::prelude::ThreadRng;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

const CHANGED_ELEMENTS: usize = 10;

fn main() {
    let all_arguments: Vec<String> = env::args().collect();
    let number_of_copies: u64;

    let mut files_to_check: Vec<PathBuf> = Vec::new();

    if all_arguments.len() >= 3 {
        let thing_to_check = all_arguments[1].clone();

        if !Path::new(&thing_to_check).exists() {
            println!("Path should exists {}", thing_to_check);
            process::exit(1);
        }

        let checked_thing = match fs::canonicalize(PathBuf::from(&thing_to_check)) {
            Ok(t) => t,
            Err(_) => {
                println!("Failed to open file {}", thing_to_check);
                process::exit(1);
            }
        };

        if checked_thing.is_file() {
            files_to_check.push(checked_thing);
        } else if checked_thing.is_dir() {
            let read_dir = match fs::read_dir(&checked_thing) {
                Ok(t) => t,
                Err(_) => {
                    println!("Failed to get files in folder {:?}", checked_thing);
                    process::exit(1);
                }
            };
            for entry in read_dir.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        files_to_check.push(entry.path());
                    }
                }
            }
        }

        number_of_copies = match all_arguments[2].parse::<u64>() {
            Ok(t) => t,
            Err(_) => {
                println!("Failed to parse number of copies");
                process::exit(1);
            }
        };
    } else {
        println!("You must provide file/folder and number of broken copies!");
        process::exit(1);
    }

    let process_characters = false;

    files_to_check.retain(|e| e.to_string_lossy().to_string().contains('.'));

    files_to_check.into_par_iter().for_each({
        |file| {
            let Some((full_name, file_name, extension)) = read_file_name_extension(file) else {
                return;
            };

            if process_characters {
            } else {
                process_bytes(full_name, file_name, extension, number_of_copies);
            }
        }
    });
}

fn process_bytes(original_name: String, file_name: String, extension: String, repeats: u64) {
    let data_vector: Vec<u8> = match fs::read(&original_name) {
        Ok(t) => t,
        Err(_) => {
            println!("Failed to read data from file {:?}!", original_name);
            return;
        }
    };
    let mut thread_rng = thread_rng();

    for i in 0..repeats {
        let new_file_name = format!("{file_name}{i}.{extension}");
        let mut file_handler = match OpenOptions::new().create(true).write(true).open(&new_file_name) {
            Ok(t) => t,
            Err(_) => {
                println!("Failed to create file {}", new_file_name);
                continue;
            }
        };

        let mut data = data_vector.clone();
        if data.is_empty() {
            continue;
        }

        split_content(&mut thread_rng, &mut data, 0.5);
        if data.is_empty() {
            continue;
        }

        let changed_items = min(data.len() / 5, 5);
        remove_random_items(&mut thread_rng, &mut data, changed_items);
        if data.is_empty() {
            continue;
        }

        let changed_items = min(data.len() / 5, 5);
        remove_random_items(&mut thread_rng, &mut data, changed_items);
        if data.is_empty() {
            continue;
        }

        if file_handler.write(&data).is_err() {
            println!("Failed to save data to file {}", new_file_name);
            continue;
        }
    }
}
// fn process_general<T>(data: Vec<T>, file_name: String, extension: String, repeats: u64) {
//
// }

fn split_content<T>(rng: &mut ThreadRng, data: &mut Vec<T>, probability: f32) {
    let idx = get_random_idx(rng, data.as_slice());
    let size = if rng.gen_range(0.0..100.0) < probability { idx } else { data.len() };
    data.truncate(size)
}

fn remove_random_items<T>(rng: &mut ThreadRng, data: &mut Vec<T>, item_number: usize) {
    for _ in 0..item_number {
        let idx = get_random_idx(rng, data.as_slice());
        data.remove(idx);
    }
}

fn modify_random_bytes<T>(rng: &mut ThreadRng, data: &mut Vec<T>, item_number: usize)
where
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    for _ in 0..item_number {
        let idx = get_random_idx(rng, data.as_slice());
        data[idx] = rng.gen::<T>();
    }
}

fn get_random_idx<T>(rng: &mut ThreadRng, data: &[T]) -> usize {
    rng.gen_range(0..data.len())
}

fn read_file_name_extension(file: PathBuf) -> Option<(String, String, String)> {
    let full_name = match file.to_str() {
        Some(k) => k.to_string(),
        None => {
            println!("File not have valid UTF-8 name {:?}!", file);
            return None;
        }
    };
    let dot_index = match full_name.rfind('.') {
        Some(t) => t,
        None => {
            println!("File {} doesn't contains a required dot", full_name);
            return None;
        }
    };
    Some((full_name.clone(), full_name[..dot_index].to_string(), full_name[(dot_index + 1)..].to_string()))
}
