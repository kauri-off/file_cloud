use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use image::io::Reader as ImageReader;
use image::{GenericImageView, Rgb, RgbImage};
use chacha20::cipher::{KeyIvInit, StreamCipher};
use sha2::Digest;
use rand::Rng;
use sha2::Sha256;

const PART_SIZE: u32 = 18 * 1024 * 1024 - 3; // 18 mb
const BLOCK_SIZE: usize = 1024 * 1024; // 1mb

fn get_random_string(len: u32) -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789";
    let mut rng = rand::thread_rng();

    let random_string: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect();

    random_string
}

fn min<T: std::cmp::PartialOrd>(x: T, y: T) -> T {
    if x < y {
        x
    } else {
        y
    }
}

pub fn split_file(file_path: &Path, folder: &Path, cinfo: &CryptoInfo) -> io::Result<Vec<String>> {
    if !folder.exists() {
        fs::create_dir(folder)?;
    }

    let mut files = Vec::new();

    if !file_path.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Not file"));
    }

    let input_file = File::open(file_path)?;
    let mut reader = BufReader::new(input_file);

    let mut bytes_read: u32 = 0;

    // Открываем первый файл для записи
    let file_name = format!("{}.png", get_random_string(16));

    files.push(file_name.clone());
    let mut part_path = folder.join(file_name);
    let part = File::create(&part_path)?;
    let mut writer = BufWriter::new(part);


    loop {
        // Читаем данные и записываем в текущий файл части
        let read = append_crypto(&mut reader, &mut writer, min(BLOCK_SIZE, (PART_SIZE-bytes_read) as usize), cinfo)?;

        // Если достигли конца файла, выходим из цикла
        if read == 0 {
            writer.flush()?;
            bin2img(&part_path)?;
            break;
        }

        bytes_read += read as u32;

        // Если достигли лимита, создаем новый файл части
        if bytes_read >= PART_SIZE {
            bytes_read = 0;

            writer.flush()?;
            bin2img(&part_path)?;

            let file_name = format!("{}.png", get_random_string(16));
            files.push(file_name.clone());

            part_path = folder.join(file_name);
            let part = File::create(&part_path)?;
            writer = BufWriter::new(part);
        }
    }

    Ok(files)
}

pub fn bin2img(file: &Path) -> io::Result<()> {
    let file_open = File::open(file)?;
    let file_size = file_open.metadata()?.size();
    let mut reader = BufReader::new(file_open);

    let mut pixel: [u8; 3] = [0; 3];

    let size = get_scale(file_size);

    let mut img = RgbImage::new(size[0], size[1]);

    'outer: for i in 0..size[0] {
        for j in 0..size[1] {
            match reader.read(&mut pixel) {
                Ok(3) => img.put_pixel(i, j, Rgb(pixel)),
                Ok(0) => break 'outer,
                Ok(n) => {
                    reader.read(&mut pixel[n..])?;
                    img.put_pixel(i, j, Rgb(pixel));
                },
                Err(e) => return Err(e.into()),
            };
        }
    }

    img.put_pixel(size[0]-1, size[1]-1, Rgb([(file_size%3).try_into().unwrap(), 0, 0]));

    fs::remove_file(file)?;
    img.save_with_format(&file, image::ImageFormat::Png).unwrap();

    Ok(())
}

pub fn img2bin(file: &Path) -> io::Result<()> {
    let temp_path_str = format!("{}.temp", get_random_string(16));
    let temp_path = Path::new(&temp_path_str);

    let temp_file = File::create(temp_path)?;
    let mut writer = BufWriter::new(temp_file);

    let mut img = ImageReader::open(file)?;
    img.set_format(image::ImageFormat::Png);
    let img = img.decode().unwrap();

    let width = img.width();
    let height = img.height();

    let meta = img.get_pixel(width-1, height-1)[0];

    'outer: for i in 0..width {
        for j in 0..height {
            let pixel = img.get_pixel(i, j);

            if i == width-1 && j == height - 1 {
                break 'outer;
            } else if i == width-1 && j == height - 2 {
                match meta {
                    0 => writer.write_all(&[pixel[0], pixel[1], pixel[2]])?,
                    1 => writer.write_all(&[pixel[0]])?,
                    2 => writer.write_all(&[pixel[0], pixel[1]])?,
                    _n => println!("How")
                }
            } else {
                writer.write_all(&[pixel[0], pixel[1], pixel[2]])?;
            }
        }
    }

    writer.flush()?;
    fs::remove_file(file)?;
    fs::copy(temp_path, file)?;
    fs::remove_file(temp_path)?;

    Ok(())
}

fn get_scale(size: u64) -> [u32; 2] {
    let size = (size as f64 / 3f64).ceil() as i32 + 1; // Pixels // +1 Meta
    let factors = factorize(size as u64);

    // [1, 2, 3, 4, 5, 6, 7] to [1, 3, 5, 7] and [2, 4, 6]
    let mut width = 1;
    let mut height = 1;

    let mut flag = true;

    for i in 0..factors.len() {
        if flag {
            width *= factors.get(i).unwrap();
        } else {
            height *= factors.get(i).unwrap();
        }

        flag = !flag;
    }


    [width as u32, height as u32]
}

fn factorize(mut n: u64) -> Vec<u64> {
    let mut ret = Vec::new();
    let mut divisor = 2;
    
    while n > 1 {
        if n % divisor == 0 {
            ret.push(divisor);
            n /= divisor;
        } else {
            divisor += 1;
        }
    }
    ret
}


pub fn concat_files(files: Vec<&Path>, file: &Path, cinfo: &CryptoInfo) -> io::Result<()> {
    let concat_file = File::create(file)?;
    let mut writer = BufWriter::new(concat_file);

    // Объединяем все части файлов в один
    for part in files {
        let temp_path_str = format!("{}.temp", get_random_string(16));
        let temp_path = Path::new(&temp_path_str);
        
        fs::copy(part, temp_path)?;
        img2bin(temp_path)?;
        append_all_file_crypto(temp_path, &mut writer, cinfo)?;
        fs::remove_file(temp_path)?;
    }

    Ok(())
}

fn append_all_file_crypto(part: &Path, file: &mut BufWriter<File>, cinfo: &CryptoInfo) -> io::Result<()> {
    let part_file = File::open(part)?;
    let mut reader = BufReader::new(part_file);

    // Читаем данные из части файла и записываем в конечный файл
    loop {
        if append_crypto(&mut reader, file, BLOCK_SIZE, cinfo)? == 0 {
            break;
        }
    }

    Ok(())
}

/// Возвращает количество прочитанных байт
fn append_crypto(from: &mut BufReader<File>, to: &mut BufWriter<File>, size: usize, cinfo: &CryptoInfo) -> io::Result<usize> {
    let mut buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    let read = from.read(&mut buf[..size])?;

    let mut cipher = cinfo.get_cipher();
    cipher.apply_keystream(&mut buf[..read]);

    if read > 0 {
        to.write_all(&buf[..read])?;
    }

    Ok(read)
}

fn _append(from: &mut BufReader<File>, to: &mut BufWriter<File>) -> io::Result<usize> {
    let mut buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    let read = from.read(&mut buf)?;

    if read > 0 {
        to.write_all(&buf[..read])?;
    }

    Ok(read)
}

pub struct CryptoInfo {
    key: [u8; 32],
    iv: [u8; 12]
}

impl CryptoInfo {
    pub fn from_cli() -> Self {
        let mut key = String::new();
        eprint!("Enter key: ");
        io::stdin().read_line(&mut key).expect("Error reading line");

        CryptoInfo::from_string(key)
    }

    pub fn from_string(key: String) -> Self {
        let key = key.trim().as_bytes().to_vec();

        let mut hasher = Sha256::new();
        hasher.update(&key);

        let mut key = [0u8; 32];
        key.copy_from_slice(&hasher.finalize().to_vec());

        let mut iv = [0u8; 12];
        iv.copy_from_slice(&key[..12]);

        CryptoInfo { key, iv }
    }

    fn get_cipher(&self) -> chacha20::cipher::StreamCipherCoreWrapper<chacha20::ChaChaCore<chacha20::cipher::typenum::UInt<chacha20::cipher::typenum::UInt<chacha20::cipher::typenum::UInt<chacha20::cipher::typenum::UInt<chacha20::cipher::typenum::UTerm, chacha20::cipher::consts::B1>, chacha20::cipher::consts::B0>, chacha20::cipher::consts::B1>, chacha20::cipher::consts::B0>>> {
        let key = self.key.as_slice();
        let iv = self.iv.as_slice();
        let cipher = chacha20::ChaCha20::new(key.into(), iv.into());

        return cipher;
    }
}