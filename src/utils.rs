use std::{fs, path::Path};

use image::DynamicImage;

/// Strip path and extension. Return basename.
///
/// # Arguments
///
/// * `filename` - The full path to the file.
///
/// # Returns
///
/// A `String` containing the basename of the file (without path and extension).
pub fn get_basename(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("")
        .to_string()
}

/// Open all images in a directory. Return a vector of `DynamicImage` instances.
///
/// # Arguments
///
/// * `directory` - The path to the directory containing the images.
///
/// # Returns
///
/// A `Result` containing a vector of `DynamicImage` instances or an error.
///
/// # Errors
///
/// Returns an error if the directory cannot be read or if any image fails to open.
pub fn open_images(directory: &Path) -> Result<Vec<DynamicImage>, Box<dyn std::error::Error>> {
    let mut images = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            match image::open(&path) {
                Ok(img) => images.push(img),
                Err(err) => return Err(Box::new(err)),
            }
        }
    }

    Ok(images)
}

/// Derive the number of columns and rows from filenames.
///
/// # Arguments
///
/// * `filenames` - A slice of filenames.
///
/// # Returns
///
/// A tuple `(num_columns, num_rows)` representing the number of columns and rows.
pub fn get_columns_rows(filenames: &[String]) -> (i32, i32) {
    let mut tiles = Vec::new();

    for filename in filenames {
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if let Some(last_part) = stem.get(stem.len().saturating_sub(5)..) {
            if let Some((row, column)) = last_part.split_once('_') {
                if let (Ok(row), Ok(column)) = (row.parse::<i32>(), column.parse::<i32>()) {
                    tiles.push((row, column));
                }
            }
        }
    }

    let rows: Vec<i32> = tiles.iter().map(|pos| pos.0).collect();
    let columns: Vec<i32> = tiles.iter().map(|pos| pos.1).collect();

    let num_rows = rows.iter().max().copied().unwrap_or(0);
    let num_columns = columns.iter().max().copied().unwrap_or(0);

    (num_columns, num_rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_columns_rows_valid_filenames() {
        let filenames = vec![
            "tile_01_01.png".to_string(),
            "tile_01_02.png".to_string(),
            "tile_02_01.png".to_string(),
            "tile_02_02.png".to_string(),
        ];

        let (num_columns, num_rows) = get_columns_rows(&filenames);
        assert_eq!(num_columns, 2);
        assert_eq!(num_rows, 2);
    }

    #[test]
    fn test_get_columns_rows_missing_parts() {
        let filenames = vec![
            "tile_01_01.png".to_string(),
            "tile_01.png".to_string(), // Missing column
            "tile_02_01.png".to_string(),
        ];

        let (num_columns, num_rows) = get_columns_rows(&filenames);
        assert_eq!(num_columns, 1);
        assert_eq!(num_rows, 2);
    }

    #[test]
    fn test_get_columns_rows_empty_filenames() {
        let filenames: Vec<String> = vec![];
        let (num_columns, num_rows) = get_columns_rows(&filenames);
        assert_eq!(num_columns, 0);
        assert_eq!(num_rows, 0);
    }

    #[test]
    fn test_open_images_valid_directory() {
        let test_dir = Path::new("test_images");
        fs::create_dir_all(test_dir).unwrap();

        // Create dummy images
        let img1 = image::DynamicImage::new_rgb8(100, 100);
        let img2 = image::DynamicImage::new_rgb8(200, 200);
        img1.save(test_dir.join("image1.png")).unwrap();
        img2.save(test_dir.join("image2.png")).unwrap();

        let images = open_images(test_dir).unwrap();
        assert_eq!(images.len(), 2);

        // Clean up
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_open_images_invalid_directory() {
        let test_dir = Path::new("non_existent_directory");
        let result = open_images(test_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_basename() {
        assert_eq!(get_basename("/path/to/image.png"), "image");
        assert_eq!(get_basename("image.jpeg"), "image");
        assert_eq!(get_basename("/path/to/image"), "image");
        assert_eq!(get_basename("/path/to/.hiddenfile"), ".hiddenfile");
        assert_eq!(get_basename(""), "");
    }
}
