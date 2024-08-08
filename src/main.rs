extern crate winapi;

use std::ffi::CString;
use std::fs::{self, File};
use std::io::{Error, Read};
use std::ptr::null_mut;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ};
use winapi::um::winnt::{GENERIC_READ, HANDLE, IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE, PAGE_READONLY, IMAGE_FILE_MACHINE_I386, IMAGE_FILE_MACHINE_AMD64};

const D: u64 = 256;

fn rk_search(pat: &str, txt: &[u8], q: u64) {
    let m = pat.len();
    let n = txt.len();
    let mut p = 0; // hash value for pattern
    let mut t = 0; // hash value for text
    let mut h = 1;

    // The value of h would be "pow(d, M-1)%q"
    for _ in 0..m-1 {
        h = (h * D) % q;
    }

    // Calculate the hash value of pattern and first window of text
    for i in 0..m {
        p = (D * p + pat.as_bytes()[i] as u64) % q;
        t = (D * t + txt[i] as u64) % q;
    }

    // Slide the pattern over text one by one
    for i in 0..=(n - m) {
        if p == t {
            // Check for characters one by one
            let mut j = 0;
            while j < m && txt[(i + j) as usize] == pat.as_bytes()[j] {
                j += 1;
            }

            // if p == t and pat[0...M-1] = txt[i, i+1, ... i+M-1]
            if j == m {
                println!("Pattern found at index {}", i);
            }
        }

        // Calculate hash value for next window of text:
        // Remove leading digit, add trailing digit
        if i < n - m {
            let txt_i = txt[i as usize] as u64;
            let txt_i_m = txt[(i + m) as usize] as u64;
            
            // Ensure that t - txt_i * h does not overflow
            let new_t = (D * (t + q - (txt_i * h) % q) + txt_i_m) % q;

            // Update t
            t = new_t;
        }
    }
}

fn read_all_bytes(path: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn list_files(path: &str) -> Result<Vec<String>, Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.push(entry.path().display().to_string());
        }
    }
    Ok(files)
}

fn get_nt_header_signature(file_path: &str) -> Result<String, Error> {
    unsafe {
        let file_name = CString::new(file_path).unwrap();
        let file_handle = CreateFileA(
            file_name.as_ptr(),
            GENERIC_READ,
            0,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );

        if file_handle == INVALID_HANDLE_VALUE {
            return Err(Error::last_os_error());
        }

        let mapping_handle = CreateFileMappingW(
            file_handle,
            null_mut(),
            PAGE_READONLY,
            0,
            0,
            null_mut(),
        );

        if mapping_handle == null_mut() {
            CloseHandle(file_handle);
            return Err(Error::last_os_error());
        }

        let base_address = MapViewOfFile(
            mapping_handle,
            FILE_MAP_READ,
            0,
            0,
            0,
        );

        if base_address == null_mut() {
            CloseHandle(mapping_handle);
            CloseHandle(file_handle);
            return Err(Error::last_os_error());
        }

        let dos_header = &*(base_address as *const IMAGE_DOS_HEADER);
        if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
            UnmapViewOfFile(base_address);
            CloseHandle(mapping_handle);
            CloseHandle(file_handle);
            return Err(Error::from_raw_os_error(87)); // ERROR_INVALID_PARAMETER
        }

        let nt_headers = &*((base_address as *const u8).offset(dos_header.e_lfanew as isize) as *const IMAGE_NT_HEADERS64);
        if nt_headers.Signature != IMAGE_NT_SIGNATURE {
            UnmapViewOfFile(base_address);
            CloseHandle(mapping_handle);
            CloseHandle(file_handle);
            return Err(Error::from_raw_os_error(87)); // ERROR_INVALID_PARAMETER
        }

        let file_header = &nt_headers.FileHeader;
        let machine = file_header.Machine;
        if machine != IMAGE_FILE_MACHINE_I386 && machine != IMAGE_FILE_MACHINE_AMD64 {
            UnmapViewOfFile(base_address);
            CloseHandle(mapping_handle);
            CloseHandle(file_handle);
            return Err(Error::from_raw_os_error(87)); // ERROR_INVALID_PARAMETER
        }

        let signature = nt_headers.Signature.to_le_bytes();
        let nt_signature_str = signature.iter().map(|b| *b as char).collect();

        UnmapViewOfFile(base_address);
        CloseHandle(mapping_handle);
        CloseHandle(file_handle);

        Ok(nt_signature_str)
    }
}

fn main() -> Result<(), Error> {
    // Start the timer
    let start_time = Instant::now();

    println!("Enter the folder directory:");
    let mut dir_path = String::new();
    std::io::stdin().read_line(&mut dir_path)?;
    let dir_path = dir_path.trim();

    println!("Enter the directory of the CSV file:");
    let mut csv_path = String::new();
    std::io::stdin().read_line(&mut csv_path)?;
    let csv_path = csv_path.trim();

    let file_paths = list_files(dir_path)?;
    let csv_data = read_all_bytes(csv_path)?;

    // Create a progress bar
    let pb = ProgressBar::new(file_paths.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg} [{elapsed_precise}] [{bar:40}] {percent}%")
        .progress_chars("#>-"));

    for entry in file_paths {
        let file_bytes = read_all_bytes(&entry)?;
        if file_bytes.len() >= 2 && &file_bytes[0..2] == b"MZ" {
            match get_nt_header_signature(&entry) {
                Ok(nt_signature) => {
                    println!();
                    rk_search(&nt_signature, &csv_data, 7);
                    println!("{} \nNT header signature found (ASCII): {}", entry, nt_signature);
                    println!();
                }
                Err(_) => {
                    println!("{} is not a valid PE file or has an unsupported architecture.", entry);
                }
            }
        } else {
            println!("{} is not a valid PE file.", entry);
        }

        // Update the progress bar
        pb.inc(1);
    }

    // Finish the progress bar
    pb.finish_with_message("Done");

    // Print elapsed time
    println!("Elapsed time: {:?}", start_time.elapsed());

    Ok(())
}
