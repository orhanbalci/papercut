pub mod tile;
pub mod utils;

use image::{DynamicImage, GenericImage, RgbaImage};
use std::{
    env, f64, fs,
    path::{Path, PathBuf},
    ptr::read_volatile,
};
pub use tile::*;
pub use utils::*;

const SPLIT_LIMIT: u32 = 99;

/// Calculate the number of columns and rows required to divide an image
/// into `n` parts.
///
/// Returns a tuple of integers in the format (num_columns, num_rows)
///
/// # Examples
///
/// ```
/// use papercut::calc_columns_rows;
///
/// let (columns, rows) = calc_columns_rows(5);
/// assert_eq!(columns, 3);
/// assert_eq!(rows, 2);
/// ```
pub fn calc_columns_rows(n: u32) -> (u32, u32) {
    let num_columns = f32::ceil(f32::sqrt(n as f32)) as u32;
    let num_rows = f32::ceil(n as f32 / num_columns as f32) as u32;
    dbg!(n, num_columns, num_rows, n as f32 / num_columns as f32);
    (num_columns, num_rows)
}

/// Calculate the combined size of tiles.
///
/// # Arguments
///
/// * `tiles` - A slice of tiles, where each tile has an `image` with a `size`
///
/// # Returns
///
/// A tuple `(width, height)` representing the combined size of the tiles.
pub fn get_combined_size(tiles: &[tile::Tile]) -> (u32, u32) {
    // TODO: Refactor calculating layout to avoid repetition.
    let (columns, rows) = calc_columns_rows(tiles.len() as u32);
    (
        tiles[0].image.width() * columns,
        tiles[0].image.height() * rows,
    )
}

/// Basic sanity checks prior to performing a split.
///
/// # Arguments
///
/// * `number_tiles` - The number of tiles to split the image into.
///
/// # Errors
///
/// Returns a `ValueError` if `number_tiles` is not an integer, less than 2, or greater than the tile limit.
pub fn validate_image(number_tiles: u32) -> Result<u32, String> {
    const TILE_LIMIT: u32 = 99 * 99;
    // Check if the number of tiles is within the valid range
    if !(2..=TILE_LIMIT).contains(&number_tiles) {
        return Err(format!(
            "Number of tiles must be between 2 and {} (you asked for {}).",
            TILE_LIMIT, number_tiles
        ));
    }

    Ok(number_tiles)
}

/// Basic checks for columns and rows values.
///
/// # Arguments
///
/// * `col` - The number of columns.
/// * `row` - The number of rows.
///
/// # Errors
///
/// Returns a `ValueError` if `col` or `row` is not an integer, is out of range, or if both are 1.
pub fn validate_image_col_row(col: u32, row: u32) -> Result<(u32, u32), String> {
    // Check if `col` and `row` are within the valid range
    if col < 1 || row < 1 || col > SPLIT_LIMIT || row > SPLIT_LIMIT {
        return Err(format!(
            "Number of columns and rows must be between 1 and {} (you asked for rows: {} and col: {}).",
            SPLIT_LIMIT, row, col
        ));
    }

    // Check if both `col` and `row` are 1
    if col == 1 && row == 1 {
        return Err("There is nothing to divide. You asked for the entire image.".to_string());
    }

    Ok((col, row))
}

/// Split an image into a specified number of tiles.
///
/// # Arguments
///
/// * `filename` - The filename of the image to split.
/// * `number_tiles` - The number of tiles required.
/// * `col` - Number of columns (optional).
/// * `row` - Number of rows (optional).
/// * `save` - Whether or not to save tiles to disk.
/// * `decompression_bomb_warning` - Whether to suppress Pillow DecompressionBombWarning.
///
/// # Returns
///
/// A vector of `Tile` instances.
pub fn slice(
    filename: &str,
    number_tiles: Option<u32>,
    col: Option<u32>,
    row: Option<u32>,
    save: bool,
) -> Result<Vec<Tile>, String> {
    let relative_path = Path::new(filename);
    let full_path = relative_path
        .canonicalize()
        .expect("Failed to canonicalize path");
    // Open the image
    let mut im = image::open(&full_path)
        .map_err(|_| format!("can not open image {}", full_path.to_str().unwrap()))?;
    let (im_w, im_h) = (im.width(), im.height());

    let (columns, rows) = if let Some(number_tiles) = number_tiles {
        validate_image(number_tiles)?;
        calc_columns_rows(number_tiles)
    } else if let (Some(col), Some(row)) = (col, row) {
        validate_image_col_row(col, row)?;
        (col, row)
    } else {
        return Err("Invalid tile configuration.".to_string());
    };
    let tile_w = im_w / columns;
    let tile_h = im_h / rows;

    let mut tiles = Vec::new();
    let mut number = 1;

    for pos_y in (0..im_h).step_by(tile_h as usize) {
        for pos_x in (0..im_w).step_by(tile_w as usize) {
            if pos_x + tile_w > im_w || pos_y + tile_h > im_h {
                continue;
            }
            let area = (
                pos_x,
                pos_y,
                u32::min(pos_x + tile_w, im_w),
                u32::min(pos_y + tile_h, im_h),
            );
            let image = im.crop(area.0, area.1, area.2 - area.0, area.3 - area.1);
            let position = ((pos_x / tile_w) as i32 + 1, (pos_y / tile_h) as i32 + 1);
            let coords = (pos_x as i32, pos_y as i32);
            let tile = Tile::new(image, number, position, coords, None);
            tiles.push(tile);
            number += 1;
        }
    }

    if save {
        let prefix = get_basename(filename);
        let directory = Path::new(filename)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        save_tiles(&mut tiles, &prefix, Some(directory), "png")
            .map_err(|_| "can not save tiles")?;
    }

    Ok(tiles)
}

/// Write image files to disk. Create specified folder(s) if they
/// don't exist. Returns a vector of `Tile` instances.
///
/// # Arguments
///
/// * `tiles` - A slice of `Tile` objects to save.
/// * `prefix` - Filename prefix of saved tiles.
/// * `directory` - Directory to save tiles. Created if non-existent.
/// * `format` - Format of the saved tiles.
///
/// # Returns
///
/// A vector of `Tile` instances.
///
/// # Errors
///
/// Returns an error if saving any tile fails.
pub fn save_tiles(
    tiles: &mut [Tile],
    prefix: &str,
    directory: Option<&Path>,
    format: &str,
) -> Result<Vec<Tile>, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let dir = directory.unwrap_or(current_dir.as_path());

    // Ensure the directory exists
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    for tile in tiles.iter_mut() {
        let filename = tile.generate_filename(Some(dir), prefix, format, true);
        tile.save(Some(filename), format)?;
    }

    Ok(tiles.to_vec())
}

/// Determine column and row position for a filename.
///
/// # Arguments
///
/// * `filename` - The filename to extract column and row information.
///
/// # Returns
///
/// A tuple `(column, row)` where both values are zero-based indices.
///
/// # Errors
///
/// Returns an error if the filename format is invalid.
pub fn get_image_column_row(filename: &str) -> Result<(i32, i32), String> {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;

    if let Some(last_part) = stem.get(stem.len().saturating_sub(5)..) {
        if let Some((row, column)) = last_part.split_once('_') {
            if let (Ok(row), Ok(column)) = (row.parse::<i32>(), column.parse::<i32>()) {
                return Ok((column - 1, row - 1)); // Convert to zero-based indices
            }
        }
    }

    Err("Invalid filename format for extracting column and row".to_string())
}

/// Open all images in a directory. Return a vector of `Tile` instances.
///
/// # Arguments
///
/// * `directory` - The path to the directory containing the images.
///
/// # Returns
///
/// A `Result` containing a vector of `Tile` instances or an error.
pub fn open_images_in(directory: &Path) -> Result<Vec<Tile>, Box<dyn std::error::Error>> {
    let files: Vec<PathBuf> = fs::read_dir(directory)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let filename = path.file_name()?.to_str()?;
            if filename.contains('_') && !filename.starts_with("joined") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let mut tiles = Vec::new();

    if !files.is_empty() {
        for (i, file) in files.iter().enumerate() {
            let pos = get_image_column_row(file.file_name().unwrap().to_str().unwrap())?;
            let im = image::open(file)?;

            let mut position_xy = [0, 0];
            position_xy[0] = pos.0 * im.width() as i32;
            position_xy[1] = pos.1 * im.height() as i32;

            tiles.push(Tile::new(
                im,
                (i + 1) as i32,
                pos,
                (position_xy[0], position_xy[1]),
                Some(file.clone()),
            ));
        }
    }

    Ok(tiles)
}

/// Combine tiles into a single image.
///
/// # Arguments
///
/// * `tiles` - A slice of `Tile` instances.
/// * `width` - Optional, width of the combined image.
/// * `height` - Optional, height of the combined image.
///
/// # Returns
///
/// A `DynamicImage` instance representing the combined image.
pub fn join(tiles: &[Tile], width: u32, height: u32) -> Result<DynamicImage, String> {
    // Determine the size of the combined image
    let (combined_width, combined_height) = if width > 0 && height > 0 {
        (width, height)
    } else {
        get_combined_size(tiles)
    };

    // Create a new blank RGBA image
    let im = RgbaImage::new(combined_width, combined_height);
    let mut target_image = DynamicImage::ImageRgba8(im);
    // Iterate over tiles and paste them into the combined image
    for tile in tiles {
        let coords = (tile.coords.0 as u32, tile.coords.1 as u32);
        let sub_image = tile.image.to_rgba8(); // Convert the tile image to RgbaImage
        target_image
            .copy_from(&sub_image, coords.0, coords.1)
            .map_err(|_| "can not copy from tile")?
    }

    Ok(target_image)
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, RgbaImage};

    use crate::tile::Tile;

    use super::*;

    #[test]
    fn test_calc_columns_rows() {
        assert_eq!(calc_columns_rows(1), (1, 1));
        assert_eq!(calc_columns_rows(2), (2, 1));
        assert_eq!(calc_columns_rows(3), (2, 2));
        assert_eq!(calc_columns_rows(4), (2, 2));
        assert_eq!(calc_columns_rows(5), (3, 2));
        assert_eq!(calc_columns_rows(6), (3, 2));
        assert_eq!(calc_columns_rows(7), (3, 3));
        assert_eq!(calc_columns_rows(8), (3, 3));
        assert_eq!(calc_columns_rows(9), (3, 3));
    }

    fn create_dummy_tile(width: u32, height: u32, number: i32) -> Tile {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(width, height));
        Tile::new(image, number, (0, 0), (0, 0), None)
    }

    #[test]
    fn test_get_combined_size_single_tile() {
        let tiles = vec![create_dummy_tile(100, 200, 1)];
        let combined_size = get_combined_size(&tiles);
        assert_eq!(combined_size, (100, 200));
    }

    #[test]
    fn test_get_combined_size_multiple_tiles() {
        let tiles = vec![
            create_dummy_tile(100, 200, 1),
            create_dummy_tile(100, 200, 2),
            create_dummy_tile(100, 200, 3),
            create_dummy_tile(100, 200, 4),
        ];
        let combined_size = get_combined_size(&tiles);
        assert_eq!(combined_size, (200, 400));
    }

    #[test]
    fn test_get_combined_size_non_square_layout() {
        let tiles = vec![
            create_dummy_tile(50, 50, 1),
            create_dummy_tile(50, 50, 2),
            create_dummy_tile(50, 50, 3),
        ];
        let combined_size = get_combined_size(&tiles);
        assert_eq!(combined_size, (100, 100));
    }

    #[test]
    fn test_get_combined_size_large_number_of_tiles() {
        let tiles = (0..16)
            .map(|i| create_dummy_tile(10, 20, i))
            .collect::<Vec<_>>();
        let combined_size = get_combined_size(&tiles);
        assert_eq!(combined_size, (40, 80));
    }
}
