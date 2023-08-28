use std::cmp::min;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

mod cli;

fn main() {
    cli::parse_cli();

    let (files_to_check, output_path, number_of_broken_files, character_mode, special_words) = cli::parse_cli();
    let all_broken_files_to_create = files_to_check.len() * number_of_broken_files as usize;
    let initial_files_number = files_to_check.len();
    println!("Collected {} files to check, creating {} broken files ({} per file)", initial_files_number, all_broken_files_to_create, number_of_broken_files);
    println!("Using character_mode - {character_mode}, used special words(only works with character_mode) - {special_words:?}({} items)", special_words.len());

    let atomic_counter = AtomicU32::new(0);

    files_to_check.into_par_iter().for_each({
        |full_name| {
            let idx = atomic_counter.fetch_add(1, Ordering::Relaxed);
            if idx % 100 == 0 {
                println!("Processed {}/{} broken files", idx, initial_files_number);
            }

            let Some((file_name, extension)) = read_file_name_extension(&full_name) else {
                return;
            };

            if character_mode {
                process_file_characters(full_name, file_name, &output_path, extension, number_of_broken_files as u64, &special_words);
            } else {
                process_file_bytes(full_name, file_name, &output_path, extension, number_of_broken_files as u64);
            }
        }
    });
}

fn process_file_bytes(original_name: String, file_name: String, output_path: &str, extension: String, repeats: u64) {
    let data_vector: Vec<u8> = match fs::read(&original_name) {
        Ok(t) => t,
        Err(_) => {
            println!("Failed to read data from file {:?}!", original_name);
            return;
        }
    };

    let mut thread_rng = thread_rng();

    for idx in 0..repeats {
        let new_file_name = format!("{output_path}/{file_name}{idx}.{extension}");
        let mut file_handler = match OpenOptions::new().create(true).write(true).open(&new_file_name) {
            Ok(t) => t,
            Err(_) => {
                println!("Failed to create file {}", new_file_name);
                return;
            }
        };
        let Some(mut data) = process_single_general(&mut thread_rng, &data_vector) else {
            continue;
        };

        if thread_rng.gen_bool(0.5) {
            let items_random = thread_rng.gen_range(1..6);
            let changed_items = min(data.len() / 5, items_random);
            modify_random_items(&mut thread_rng, &mut data, changed_items);
            if data.is_empty() {
                continue;
            }
        }

        if file_handler.write_all(&data).is_err() {
            println!("Failed to save data to file {}", new_file_name);
            continue;
        }
    }
}

fn process_file_characters(original_name: String, file_name: String, output_path: &str, extension: String, repeats: u64, special_words: &[String]) {
    let data_vector: Vec<char> = match fs::read_to_string(&original_name) {
        Ok(t) => t.chars().collect(),
        Err(_) => {
            println!("Failed to read data from file {:?}!", original_name);
            return;
        }
    };

    let mut thread_rng = thread_rng();

    for idx in 0..repeats {
        let new_file_name = format!("{output_path}/{file_name}{idx}.{extension}");
        let mut file_handler = match OpenOptions::new().create(true).write(true).open(&new_file_name) {
            Ok(t) => t,
            Err(_) => {
                println!("Failed to create file {}", new_file_name);
                return;
            }
        };
        let Some(mut data) = process_single_general(&mut thread_rng, &data_vector) else {
            continue;
        };

        if thread_rng.gen_bool(0.5) {
            let items_random = thread_rng.gen_range(1..6);
            let changed_items = min(data.len() / 5, items_random);
            modify_random_char_items(&mut thread_rng, &mut data, changed_items);
            if data.is_empty() {
                continue;
            }
        }

        if !special_words.is_empty() && thread_rng.gen_bool(0.5) {
            let items_random = thread_rng.gen_range(1..5);
            add_random_words(&mut thread_rng, &mut data, items_random, special_words);
            if data.is_empty() {
                continue;
            }
        }

        if file_handler.write_all(data.into_iter().collect::<String>().as_bytes()).is_err() {
            println!("Failed to save data to file {}", new_file_name);
            continue;
        }
    }
}

fn process_single_general<T>(thread_rng: &mut ThreadRng, data_vector: &[T]) -> Option<Vec<T>>
where
    T: Clone,
    rand::distributions::Standard: Distribution<T>,
{
    let mut data = data_vector.to_vec();
    if data.is_empty() {
        return None;
    }

    if thread_rng.gen_bool(0.5) {
        split_content(thread_rng, &mut data);
        if data.is_empty() {
            return None;
        }
    }

    if thread_rng.gen_bool(0.5) {
        let items_random = thread_rng.gen_range(1..6);
        let changed_items = min(data.len() / 5, items_random);
        remove_random_items(thread_rng, &mut data, changed_items);
        if data.is_empty() {
            return None;
        }
    }

    Some(data)
}

fn add_random_words(rng: &mut ThreadRng, data: &mut Vec<char>, words_number: u32, words: &[String]) {
    for _ in 0..words_number {
        let mut word = words.choose(rng).unwrap().chars().collect::<Vec<_>>();
        if rng.gen_bool(0.1) {
            word.insert(0, ' ');
        }
        if rng.gen_bool(0.01) {
            word.insert(0, '\t');
        }
        if rng.gen_bool(0.01) {
            word.insert(0, '\n');
        }
        if rng.gen_bool(0.4) {
            word.push(' ');
        }

        let idx = get_random_idx(rng, data);
        let temp_data = data[idx..].to_vec();
        data.truncate(idx);
        data.extend(word);
        data.extend(temp_data);
    }
}

fn split_content<T>(rng: &mut ThreadRng, data: &mut Vec<T>) {
    let idx = get_random_idx(rng, data.as_slice());
    data.truncate(idx)
}

fn remove_random_items<T>(rng: &mut ThreadRng, data: &mut Vec<T>, item_number: usize) {
    for _ in 0..item_number {
        let idx = get_random_idx(rng, data.as_slice());
        data.remove(idx);
    }
}

fn modify_random_items<T>(rng: &mut ThreadRng, data: &mut Vec<T>, item_number: usize)
where
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    for _ in 0..item_number {
        let idx = get_random_idx(rng, data.as_slice());
        data[idx] = rng.gen::<T>();
    }
}
fn modify_random_char_items(rng: &mut ThreadRng, data: &mut Vec<char>, item_number: usize) {
    for _ in 0..item_number {
        let idx = get_random_idx(rng, data.as_slice());
        data[idx] = rng.gen_range(0..=255).into();
    }
}

fn get_random_idx<T>(rng: &mut ThreadRng, data: &[T]) -> usize {
    rng.gen_range(0..data.len())
}

fn read_file_name_extension(full_name: &str) -> Option<(String, String)> {
    let full_name_path = Path::new(full_name);

    let Some(file_name) = full_name_path.file_stem() else {
        println!("Failed to find file name");
        return None;
    };
    let file_name = file_name.to_string_lossy().to_string();

    let Some(extension) = full_name_path.extension() else {
        println!("Failed to find extension");
        return None;
    };
    let extension = extension.to_string_lossy().to_string();

    Some((file_name, extension))
}

#[test]
fn test_add_random_words() {
    let mut rng = thread_rng();
    for _ in 0..20 {
        let mut data = "SSSSS_____LLLLLKKKKK".chars().collect::<Vec<_>>();
        assert_eq!(data.len(), 20);
        add_random_words(&mut rng, &mut data, 1, &["CZCZEK".to_string()]);
        assert_eq!(data.len(), 26);
    }
}
