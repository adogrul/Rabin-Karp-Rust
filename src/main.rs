use std::fs::{self, File};
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};

// Dosya boyutunu bulma fonksiyonu
fn get_file_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let file_size = file.seek(SeekFrom::End(0))?;
    Ok(file_size)
}

// Dosyayı okuyup bir diziye atma fonksiyonu
fn read_all_bytes<P: AsRef<Path> + Clone>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path.clone())?;
    let file_size = get_file_size(path)?;
    let mut buffer = vec![0; file_size as usize];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

// Dizin içerisindeki dosyaları listeleme fonksiyonu
fn sub_dir_list_files<P: AsRef<Path>>(dir_path: P) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(files)
}

// Arama fonksiyonu
fn search(keywords: &str, file_path: &str, q: i32, progress_bar: &ProgressBar) -> io::Result<()> {
    let pat = keywords.as_bytes();
    let m = pat.len();
    let n = get_file_size(file_path)? as usize;
    let txt = read_all_bytes(file_path)?;
    
    let mut p = 0; // Pattern hash değeri
    let mut t = 0; // Text hash değeri
    let mut h = 1;

    // h'nin değerini hesapla
    for _ in 0..m-1 {
        h = (h * 256) % q;
    }

    // Pattern ve text'in ilk penceresinin hash değerlerini hesapla
    for i in 0..m {
        p = (256 * p + pat[i] as i32) % q;
        t = (256 * t + txt[i] as i32) % q;
    }

    // Pattern'i text üzerinde kaydırarak ara
    for i in 0..=n - m {
        // Mevcut pencerenin hash değerlerini kontrol et
        if p == t {
            // Karakterleri tek tek kontrol et
            if &txt[i..i + m] == pat {
                println!("Pattern found at index {}", i);
            }
        }

        // Sonraki pencere için hash değerini hesapla
        if i < n - m {
            t = (256 * (t - (txt[i] as i32 * h)) + txt[i + m] as i32) % q;
            if t < 0 {
                t += q;
            }
        }

        // İlerleme çubuğunu güncelle
        progress_bar.inc(1);
    }

    progress_bar.finish();
    Ok(())
}

fn main() -> io::Result<()> {
    let q = 7; // Bir asal sayı

    let mut dir_path = String::new();
    println!("klasör dizinini giriniz(Enter the folder directory): ");
    io::stdin().read_line(&mut dir_path)?;
    let dir_path = dir_path.trim();

    let mut csv_path = String::new();
    println!("csv dosyasının yolunu giriniz(Enter the csv file path): ");
    io::stdin().read_line(&mut csv_path)?;
    let csv_path = csv_path.trim();

    let dir_arr = sub_dir_list_files(dir_path)?;
    let mut keywords = Vec::new();

    let csv_file = File::open(csv_path)?;
    let reader = io::BufReader::new(csv_file);

    for line in reader.lines() {
        let line = line?;
        keywords.push(line);
    }

    let total_files = dir_arr.len() as u64;
    let progress_bar = ProgressBar::new(total_files);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    let start_time = Instant::now();

    for entry in dir_arr {
        for keyword in &keywords {
            let file_size = get_file_size(&entry)? as usize;
            let keyword_len = keyword.len();
            let iterations = file_size.saturating_sub(keyword_len) as u64;

            let pb = ProgressBar::new(iterations);
            pb.set_style(ProgressStyle::default_bar()
                .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .progress_chars("#>-"));

            pb.set_message(format!("Processing {}", entry));

            search(keyword, &entry, q, &pb)?;
        }
        progress_bar.inc(1);
    }

    progress_bar.finish_with_message("All files processed");

    let duration = start_time.elapsed();
    println!("Total duration: {:.2?} seconds", duration);

    Ok(())
}
