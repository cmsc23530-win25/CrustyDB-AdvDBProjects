use std::fs::File;
use std::io::{self, BufRead};
use std::env;

pub fn convert_file_wrapper(file_name: &str) ->  io::Result<Vec<Vec<u8>>>  {
    let mut current_dir = env::current_dir().expect("Failed to get current directory");
    current_dir.push("src");
    //current_dir.push("dictionary");
    current_dir.push("data");
    let file_name = current_dir.join(file_name);
    convert_file(file_name.to_str().expect("Failed to convert PathBuf to &str"))
}

pub fn convert_file(file_name: &str) ->  io::Result<Vec<Vec<u8>>>  {
    // Open the file
    let file = File::open(file_name)?;
    let reader = io::BufReader::new(file);

    // Create a Vec<u8> to store the ASCII byte values
    let mut ascii_values: Vec<Vec<u8>> = Vec::new();

    // Iterate over each line in the file
    for line in reader.lines() {
        ascii_values.push(convert_string(&line?));
    }
    Ok(ascii_values)
}

pub fn convert_string(input: &str) -> Vec<u8> {
    let mut ascii_values: Vec<u8> = Vec::new();
    for char in input.chars() {
        ascii_values.push(char as u8);
    }
    ascii_values
}

#[allow(dead_code)]
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2, " Need to pass file argument");
    // Path to the input text file
    let file_path = &args[1];
    let ascii_values = convert_file(file_path)?;
    // Print the resulting Vec<u8>
    println!("{:?}", ascii_values);
    Ok(())
}
