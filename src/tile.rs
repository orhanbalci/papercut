use image::DynamicImage;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Tile {
    pub image: DynamicImage,
    pub number: i32,
    pub position: (i32, i32),
    pub coords: (i32, i32), // Assuming coords is a tuple of integers
    pub filename: Option<PathBuf>,
}

impl Tile {
    pub fn new(
        image: DynamicImage,
        number: i32,
        position: (i32, i32),
        coords: (i32, i32),
        filename: Option<PathBuf>,
    ) -> Self {
        Tile {
            image,
            number,
            position,
            coords,
            filename,
        }
    }

    pub fn row(&self) -> i32 {
        self.position.0
    }

    pub fn column(&self) -> i32 {
        self.position.1
    }

    pub fn basename(&self) -> Option<String> {
        self.filename.as_ref().and_then(|f| {
            f.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
    }

    pub fn generate_filename(
        &self,
        directory: Option<&Path>,
        prefix: &str,
        format: &str,
        path: bool,
    ) -> PathBuf {
        let current_dir = env::current_dir().unwrap();
        let dir = directory.unwrap_or(current_dir.as_path());
        let ext = format.to_lowercase().replace("jpeg", "jpg");
        let filename = format!("{}_{:02}_{:02}.{}", prefix, self.column(), self.row(), ext);

        if path {
            dir.join(filename)
        } else {
            PathBuf::from(filename)
        }
    }

    pub fn save(
        &mut self,
        filename: Option<PathBuf>,
        format: &str,
    ) -> Result<(), image::ImageError> {
        let file_path =
            filename.unwrap_or_else(|| self.generate_filename(None, "tile", format, true));
        self.image.save(&file_path)?;
        self.filename = Some(file_path);
        Ok(())
    }
}

impl std::fmt::Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.filename {
            Some(filename) => write!(
                f,
                "<Tile #{} - {}>",
                self.number,
                filename.file_name().unwrap_or_default().to_string_lossy()
            ),
            None => write!(f, "<Tile #{}>", self.number),
        }
    }
}
