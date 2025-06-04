use papercut::{get_basename, save_tiles, slice};
use pico_args::Arguments;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();

    // Parse arguments

    println!("Current directory: {:?}", std::env::current_dir()?);
    let image: String = args.value_from_str("--image")?;
    let num_tiles: u32 = args.opt_value_from_str("--num-tiles")?.unwrap_or(0);
    let dir: String = args
        .opt_value_from_str("--dir")?
        .unwrap_or_else(|| "./".to_string());
    let format: String = args
        .opt_value_from_str("--format")?
        .unwrap_or_else(|| "png".to_string());
    let rows: u32 = args.opt_value_from_str("--rows")?.unwrap_or(1);
    let columns: u32 = args.opt_value_from_str("--columns")?.unwrap_or(1);

    // Validate arguments
    if num_tiles == 0 && rows == 1 && columns == 1 {
        eprintln!(
            "No operation specified. You need to either specify the number of tiles to slice \
            automatically, or specify the row and columns to customize the slice."
        );
        std::process::exit(1);
    }

    // Slice the image
    let tiles = slice(
        &image,
        if num_tiles > 0 { Some(num_tiles) } else { None },
        if columns > 1 { Some(columns) } else { None },
        if rows > 1 { Some(rows) } else { None },
        false,
    )?;

    // Save the tiles
    save_tiles(
        &mut tiles.clone(),
        &get_basename(&image),
        Some(Path::new(&dir)),
        &format,
    )?;

    Ok(())
}
