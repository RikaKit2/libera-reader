use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, Level};
use walkdir::WalkDir;


fn main() {
  let subscriber = tracing_subscriber::fmt()
    .pretty()
    .without_time()
    .compact()
    .with_file(false)
    .with_line_number(false)
    .with_thread_ids(false)
    .with_target(false)
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).unwrap();
  // println!("Enter the path to the folder to scan:");
  // let mut user_input = String::new();
  // io::stdin().read_line(&mut user_input).expect("Failed to read line");
  // let path_to_scan = user_input.trim().to_string();
  let path_to_scan = "/media/user/WDC_WD5000LPCX/other_files/my_books/favorites/Books_favorites/books-pdf";
  let out_dir = PathBuf::new().join("out");
  if !out_dir.exists() {
    std::fs::create_dir(&out_dir).expect("TODO: panic message");
  };
  let mut books_paths_from_disk: VecDeque<String> = VecDeque::new();
  for entry in WalkDir::new(path_to_scan) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() {
      let path = entry.path();
      let file_extension = path.extension().unwrap().to_str().unwrap().to_string();
      if ["pdf".to_string()].contains(&file_extension) {
        books_paths_from_disk.push_back(path.to_str().unwrap().to_string());
      }
    }
  };
  run_task(books_paths_from_disk, out_dir);
  loop {
    thread::sleep(Duration::from_secs(10));
  }
}

fn run_task(mut books_paths_on_disk: VecDeque<String>, out_dir: PathBuf) {
  let num_of_books = books_paths_on_disk.len();
  println!("num_of_books: {:?}", num_of_books);
  if num_of_books > 0 {
    let now = Instant::now();
    for book_num in 0..num_of_books {
      match books_paths_on_disk.pop_front() {
        Some(path_to_book) => {
          let img_name = out_dir.clone().join(book_num.to_string())
            .with_extension("jpeg").to_str().unwrap().to_string();
          extract_book_thumbnail(&path_to_book, img_name);
        }
        None => {}
      };
    }
    let elapsed = now.elapsed();
    println!("service uptime: {:?}", elapsed);
  }
}
fn write_img_data_to_file(img_name: &String, img_data: &mut Vec<u8>) {
  match File::create(img_name) {
    Ok(mut output_file) => {
      debug!("len of img_data{:?}", img_data.len());
      match output_file.write_all(img_data) {
        Ok(_) => { info!(img_name) }
        Err(err) => { error!("Data writing error: {:?}", err) }
      }
    }
    Err(err) => { error!("File creating error: {:?}", err) }
  };
}
fn extract_book_thumbnail(path_to_book: &String, img_name: String) {
  match mupdf::document::Document::open(path_to_book, 20) {
    Ok(doc) => {
      match doc.load_page(0) {
        Ok(page) => {
          match page.to_pixmap(0.4) {
            Ok(mut pixmap) => {
              // let mut img_data: Vec<u8> = vec![];
              // match pixmap.save_as_jpeg_to_storage(70, &mut img_data) {
              //   Ok(_) => {
              //     write_img_data_to_file(&img_name, &mut img_data)
              //   }
              //   Err(err) => { error!("jpeg extracting error: \nerr: {:?}", &err); }
              // }
              match pixmap.save_as_jpeg(70, img_name) {
                Ok(_) => {}
                Err(err) => { error!("jpeg extracting error: \nerr: {:?}", &err); }
              }
            }
            Err(err) => { error!("Pixmap creating error: {:?}", &err); }
          };
        }
        Err(err) => { error!("Page loading error: {:?}", &err); }
      }
    }
    Err(err) => { error!("Document loading error: {:?}", &err); }
  }
}

fn extract_book_thumbnail3(book_path: &String, img_name: String) {
  let mut img_data: Vec<u8> = vec![];
  let mupdf_res = mupdf::get_thumbnail_from_document2(book_path, 20, 0, 0.0, 0.4, 70, &img_name);
  info!("mupdf_res is ok: {}", mupdf_res.is_ok());
  info!("len of img_data: {}", img_data.len());
  match mupdf_res {
    Ok(_) => {
      info!("is ok");
      // write_img_data_to_file(&img_name, &mut img_data)
    }
    Err(_) => {}
  }
}
