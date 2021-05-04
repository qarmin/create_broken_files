use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

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
            for entry in read_dir {
                if let Ok(entry_data) = entry {
                    if let Ok(metadata) = entry_data.metadata() {
                        if metadata.is_file() {
                            files_to_check.push(entry_data.path());
                        }
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

    files_to_check = files_to_check.iter().filter(|e| e.to_string_lossy().to_string().contains('.')).cloned().collect();

    files_to_check.par_iter().for_each({
        |file| {
            let mut data_vector: Vec<u8>;
            data_vector = match fs::read(&file) {
                Ok(t) => t,
                Err(_) => {
                    println!("Failed to read data from file {:?}!", file);
                    return;
                }
            };

            let file_as_str = file.to_string_lossy().to_string();
            let dot_index = match file_as_str.rfind('.') {
                Some(t) => t,
                None => {
                    println!("File {} doesn't contains a required dot", file_as_str);
                    return;
                }
            };

            let mut random_values: [u8; CHANGED_ELEMENTS] = [0; CHANGED_ELEMENTS];
            let mut old_values: [u8; CHANGED_ELEMENTS] = [0; CHANGED_ELEMENTS];
            let mut random_indexes: [usize; CHANGED_ELEMENTS] = [0; CHANGED_ELEMENTS];

            for i in 0..number_of_copies {
                for j in 0..CHANGED_ELEMENTS {
                    if data_vector.len() == 0 {
                        continue;
                    }
                    let random_index = thread_rng().gen_range(0..data_vector.len());
                    let random_value = thread_rng().gen_range(0..=255u8);
                    random_indexes[j] = random_index;
                    random_values[j] = random_value;

                    old_values[j] = data_vector[random_index];

                    data_vector[random_index] = random_value;
                }

                let new_file_name: String = format!("{}{}{}", &file_as_str[..dot_index], i, &file_as_str[dot_index..]);

                let mut file_handler = match OpenOptions::new().create(true).write(true).open(&new_file_name) {
                    Ok(t) => t,
                    Err(_) => {
                        for j in 0..CHANGED_ELEMENTS {
                            data_vector[random_indexes[j]] = old_values[j];
                        }
                        println!("Failed to create file {}", new_file_name);
                        continue;
                    }
                };
                if data_vector.len() != 0 {
                    let size;
                    // Small change to randomly split
                    if thread_rng().gen_range(0..10) == 0 {
                        size = thread_rng().gen_range(0..data_vector.len());
                    } else {
                        size = data_vector.len();
                    }
                    // Normal full saving

                    if file_handler.write(&(data_vector[..size])).is_err() {
                        for j in 0..CHANGED_ELEMENTS {
                            data_vector[random_indexes[j]] = old_values[j];
                        }
                        println!("Failed to save data to file {}", new_file_name);
                        continue;
                    }

                    for j in 0..CHANGED_ELEMENTS {
                        data_vector[random_indexes[j]] = old_values[j];
                    }
                }
            }
        }
    });
}
