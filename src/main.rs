use std::cmp::min;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::cli::Cli;
use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon::prelude::*;

mod cli;

fn main() {
    cli::parse_cli();

    let (files_to_check, settings) = cli::parse_cli();
    let all_broken_files_to_create = files_to_check.len() * settings.number_of_broken_files as usize;
    let initial_files_number = files_to_check.len();
    println!("Collected {initial_files_number} files to check, creating {all_broken_files_to_create} broken files ({} per file)", settings.number_of_broken_files);
    println!(
        "Using character_mode - {}, used special words(only works with character_mode) - {:?}({} items), used connecting multiple files - {}",
        settings.character_mode,
        settings.special_words,
        settings.special_words.len(),
        settings.connect_multiple_files
    );

    let atomic_counter = AtomicU32::new(0);
    let files_to_check_clone = files_to_check.clone();

    files_to_check.into_par_iter().for_each({
        |full_name| {
            let idx = atomic_counter.fetch_add(1, Ordering::Relaxed);
            if idx % 100 == 0 {
                println!("Processed {idx}/{initial_files_number} broken files");
            }

            let Some((file_name, extension)) = read_file_name_extension(&full_name) else {
                return;
            };

            if settings.character_mode {
                process_file_characters(&full_name, &file_name, &extension, &files_to_check_clone, &settings);
            } else {
                process_file_bytes(&full_name, &file_name, &extension, &files_to_check_clone, &settings);
            }
        }
    });
}

fn process_file_bytes(original_name: &str, file_name: &str, extension: &str, files: &[String], settings: &Cli) {
    let data_vector: Vec<u8> = if let Ok(t) = fs::read(original_name) {
        t
    } else {
        println!("Failed to read data from file {original_name:?}!");
        return;
    };

    let mut thread_rng = thread_rng();

    for idx in 0..settings.number_of_broken_files {
        let new_file_name;
        loop {
            let random_u64: u64 = thread_rng.gen();
            let temp_file_name = format!("{}/{file_name}_IDX_{idx}_RAND_{random_u64}.{extension}", settings.output_path);
            if !Path::new(&temp_file_name).exists() {
                new_file_name = temp_file_name;
                break;
            }
        }

        let Some(mut data) = process_single_general(&mut thread_rng, &data_vector) else {
            continue;
        };

        if settings.connect_multiple_files {
            let additional_content = load_content_of_files(&mut thread_rng, 0.1, files);
            data.extend(additional_content);
        }

        modify_random_bytes(&mut thread_rng, 0.5, &mut data);

        if let Err(e) = fs::write(&new_file_name, &data) {
            println!("Failed to save data to file {new_file_name} - {e}");
            continue;
        }
    }
}

fn process_file_characters(original_name: &str, file_name: &str, extension: &str, files: &[String], settings: &Cli) {
    let data_vector: Vec<char> = if let Ok(t) = fs::read_to_string(original_name) {
        t.chars().collect()
    } else {
        println!("Failed to read data from file {original_name:?}!");
        return;
    };

    let mut thread_rng = thread_rng();

    for idx in 0..settings.number_of_broken_files {
        let new_file_name;
        loop {
            let random_u64: u64 = thread_rng.gen();
            let temp_file_name = format!("{}/{file_name}_IDX_{idx}_RAND_{random_u64}.{extension}", settings.output_path);
            if !Path::new(&temp_file_name).exists() {
                new_file_name = temp_file_name;
                break;
            }
        }

        let Some(mut data) = process_single_general(&mut thread_rng, &data_vector) else {
            continue;
        };

        if settings.connect_multiple_files {
            let additional_content = load_content_of_files(&mut thread_rng, 0.1, files);
            let st = String::from_utf8_lossy(&additional_content);
            data.extend(st.chars());
        }

        modify_random_char_items(&mut thread_rng, 0.5, &mut data);

        if !settings.special_words.is_empty() {
            add_random_words(&mut thread_rng, 0.5, &mut data, &settings.special_words);
        }

        if let Err(e) = fs::write(&new_file_name, data.into_iter().collect::<String>().as_bytes()) {
            println!("Failed to save data to file {new_file_name} - {e}");
            continue;
        }
    }
}

fn load_content_of_files(thread_rng: &mut ThreadRng, chance: f64, files: &[String]) -> Vec<u8> {
    assert!(!files.is_empty());
    if !thread_rng.gen_bool(chance) {
        return Vec::new();
    }

    let files_number = thread_rng.gen_range(1..8);
    let choosen_files = files.choose_multiple(thread_rng, files_number).collect::<Vec<_>>();
    choosen_files.iter().flat_map(|e| fs::read(e).unwrap_or_default()).collect()
}

fn process_single_general<T>(thread_rng: &mut ThreadRng, data_vector: &[T]) -> Option<Vec<T>>
where
    T: Clone,
    rand::distributions::Standard: Distribution<T>,
{
    if data_vector.is_empty() {
        return None;
    }
    let mut data = data_vector.to_vec();

    split_content(thread_rng, 0.2, &mut data)?;

    remove_random_items(thread_rng, 0.5, &mut data)?;

    Some(data)
}

fn add_random_words(rng: &mut ThreadRng, chance: f64, data: &mut Vec<char>, words: &[String]) {
    assert!(!data.is_empty());
    if !rng.gen_bool(chance) {
        return;
    }

    let words_number = rng.gen_range(1..5);
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
        if rng.gen_bool(0.3) {
            word.push(' ');
        }

        let idx = get_random_idx(rng, data);
        let temp_data = data[idx..].to_vec();
        data.truncate(idx);
        data.extend(word);
        data.extend(temp_data);
    }
}

fn split_content<T>(rng: &mut ThreadRng, chance: f64, data: &mut Vec<T>) -> Option<()> {
    assert!(!data.is_empty());
    if !rng.gen_bool(chance) {
        return Some(());
    }

    let idx = get_random_idx(rng, data.as_slice());
    data.truncate(idx);

    if data.is_empty() {
        None
    } else {
        Some(())
    }
}

fn remove_random_items<T>(rng: &mut ThreadRng, chance: f64, data: &mut Vec<T>) -> Option<()> {
    assert!(!data.is_empty());
    if !rng.gen_bool(chance) {
        return Some(());
    }

    let items_random = rng.gen_range(1..6);
    let item_number = min(data.len() / 5, items_random);

    for _ in 0..item_number {
        let idx = get_random_idx(rng, data.as_slice());
        data.remove(idx);
    }

    if data.is_empty() {
        None
    } else {
        Some(())
    }
}

fn modify_random_bytes(rng: &mut ThreadRng, chance: f64, data: &mut Vec<u8>) {
    assert!(!data.is_empty());
    if !rng.gen_bool(chance) {
        return;
    }

    let items_random = rng.gen_range(1..6);
    let item_number = min(data.len() / 5, items_random);
    for _ in 0..item_number {
        let idx = get_random_idx(rng, data.as_slice());
        match rng.gen_range(0..3) {
            0 => data[idx] = data[idx].overflowing_add(1).0,
            1 => data[idx] = data[idx].overflowing_sub(1).0,
            2 => data[idx] = rng.gen(),
            _ => unreachable!(),
        }
    }
}

fn modify_random_char_items(rng: &mut ThreadRng, chance: f64, data: &mut Vec<char>) {
    assert!(!data.is_empty());
    if !rng.gen_bool(chance) {
        return;
    }

    let items_random = rng.gen_range(1..6);
    let item_number = min(data.len() / 5, items_random);
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
        add_random_words(&mut rng, 1.0, &mut data, &["CZCZEK".to_string()]);
    }
}

#[test]
fn test_load_content_of_files() {
    let mut rng = thread_rng();
    let files = vec!["RRRRRRRRRRRRRRRRRRRRRRrtest.txt".to_string()];
    let _ = load_content_of_files(&mut rng, 1.0, &files);

    let new_files = (0..50).into_iter().map(|e| format!("RRRRRRRRRRRRRRRRRRRRRRrtest{e}.txt")).collect::<Vec<_>>();
    let _ = load_content_of_files(&mut rng, 1.0, &new_files);
}
