//! The `tile` module provides the `Tile` struct and associated methods for managing individual image tiles.
//! A `Tile` represents a portion of an image, along with metadata such as its position, coordinates, and filename.
//!
//! # Features
//!
//! - **Tile Representation**: Encapsulates image data and metadata for individual tiles.
//! - **Filename Generation**: Generates filenames for tiles based on their position and format.
//! - **Saving Tiles**: Saves tiles to disk in various formats.
//! - **Utility Methods**: Provides methods for accessing tile properties such as row, column, and basename.
//!
//! # Example Usage
//!
//! ```rust
//! use papercut::Tile;
//! use image::{DynamicImage, RgbaImage};
//!
//! let image = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
//! let tile = Tile::new(image, 1, (0, 0), (0, 0), None);
//! println!("{:?}", tile);
//! ```

use image::DynamicImage;
use std::env;
use std::path::{Path, PathBuf};

/// Represents a single tile of an image.
///
/// A `Tile` contains the image data, its position in the grid, pixel coordinates, and optional
#[derive(Clone)]
pub struct Tile {
    /// The image data for the tile.
    pub image: DynamicImage,
    /// The unique number assigned to the tile.
    pub number: i32,
    /// The row and column position of the tile in the grid.
    pub position: (i32, i32),
    /// The pixel coordinates of the tile in the original image.
    pub coords: (i32, i32),
    /// The filename of the tile, if it has been saved to disk.
    pub filename: Option<PathBuf>,
}

impl Tile {
    /// Creates a new `Tile` instance.
    ///
    /// # Arguments
    ///
    /// * `image` - The image data for the tile.
    /// * `number` - The unique number assigned to the tile.
    /// * `position` - The row and column position of the tile in the grid.
    /// * `coords` - The pixel coordinates of the tile in the original image.
    /// * `filename` - The filename of the tile, if it has been saved to disk.
    ///
    /// # Returns
    ///
    /// A new `Tile` instance.
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

    /// Returns the row position of the tile.
    ///
    /// # Returns
    ///
    /// The row position of the tile.
    pub fn row(&self) -> i32 {
        self.position.0
    }

    /// Returns the column position of the tile.
    ///
    /// # Returns
    ///
    /// The column position of the tile.
    pub fn column(&self) -> i32 {
        self.position.1
    }

    /// Returns the base name of the tile's filename (without path or extension).
    ///
    /// # Returns
    ///
    /// The base name of the tile's filename, or `None` if the filename is not set.
    pub fn basename(&self) -> Option<String> {
        self.filename.as_ref().and_then(|f| {
            f.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
    }

    /// Generates a filename for the tile based on its position and format.
    ///
    /// # Arguments
    ///
    /// * `directory` - The directory to save the tile. If `None`, the current working directory is used.
    /// * `prefix` - The prefix for the filename.
    /// * `format` - The format of the tile (e.g., `"png"`, `"jpg"`).
    /// * `path` - Whether to include the full path in the filename.
    ///
    /// # Returns
    ///
    /// A `PathBuf` representing the generated filename.
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

    /// Saves the tile to disk.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to save the tile. If `None`, a filename is generated using `generate_filename`.
    /// * `format` - The format of the tile (e.g., `"png"`, `"jpg"`).
    ///
    /// # Returns
    ///
    /// `Ok(())` if the tile is saved successfully, or an error if saving fails.
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
