# Rabin-Karp String Search with Progress Bar in Rust

This document provides an overview of the Rabin-Karp string search implementation in Rust, including functions for file handling, string searching, and progress reporting.

## Functions and Definitions

### `get_file_size`

**Definition:** Returns the size of a file.

**Signature:**
```rust
fn get_file_size<P: AsRef<Path>>(path: P) -> io::Result<u64>
```

### `read_all_bytes`

**Definition:** Reads the entire content of a file into a byte vector.

**Signature:**
```rust
fn read_all_bytes<P: AsRef<Path> + Clone>(path: P) -> io::Result<Vec<u8>>
```
### `sub_dir_list_files`

**Definition:** Lists all files in a specified directory.

**Signature:**
```rust
fn sub_dir_list_files<P: AsRef<Path>>(dir_path: P) -> io::Result<Vec<String>>
```

### `search`

**Definition:** Implements the Rabin-Karp string search algorithm. Updates a progress bar during the search.

**Signature:**
```rust
fn search(keywords: &str, file_path: &str, q: i32, progress_bar: &ProgressBar) -> io::Result<()>
```

### `main`

**Definition:** Coordinates reading the directory and CSV file, initializing the progress bar, and invoking the search function for each file and keyword.

**Signature:**
```rust
fn main() -> io::Result<()>
```
### Progress Bar

**Definition:** Displays a visual indication of the processing status for both the overall progress and the progress within each file.

**Signature:**
```rust
let progress_bar = ProgressBar::new(total_files);
let pb = ProgressBar::new(iterations);
```
