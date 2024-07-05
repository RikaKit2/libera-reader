use directories::ProjectDirs;

pub struct AppDirs {
  pub path_to_db: String,
  pub thumbnails_dir: String,
  pub tts_models: String,
}

impl AppDirs {
  pub fn new() -> Result<Self, Vec<std::io::Error>> {
    let mut output = vec![];
    let proj_dirs = ProjectDirs::from("com", "RikaKit", "libera-reader").unwrap();
    let data_dir = proj_dirs.data_dir().to_path_buf();

    let tts_models = data_dir.join("tts_models");
    let thumbnails_dir = data_dir.join("thumbnails");

    let path_to_db = data_dir.join("libera-reader").with_extension("db");

    let necessary_dirs = [&data_dir, &tts_models, &thumbnails_dir];
    for necessary_dir in necessary_dirs {
      if !necessary_dir.exists() {
        match std::fs::create_dir(necessary_dir) {
          Ok(_) => {}
          Err(e) => {
            output.push(e);
          }
        }
      }
    }
    if output.len() > 0 {
      Err(output)
    } else {
      Ok(Self {
        path_to_db: path_to_db.to_str().unwrap().to_string(),
        thumbnails_dir: thumbnails_dir.to_str().unwrap().to_string(),
        tts_models: tts_models.to_str().unwrap().to_string(),
      })
    }
  }
}

