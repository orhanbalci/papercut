# Papercut

Papercut is a Rust library and CLI tool for slicing and joining images. It provides functionality to split an image into tiles, save those tiles, and combine them back into a single image. This library is useful for image processing tasks such as creating image grids or splitting large images into smaller parts.

## Features

- **Slice Images**: Split an image into tiles based on the number of tiles or specified rows and columns.
- **Save Tiles**: Save the sliced tiles to disk with customizable filenames and formats.
- **Join Tiles**: Combine tiles back into a single image.
- **Validation**: Perform sanity checks on the number of tiles, rows, and columns.
- **Utilities**: Includes helper functions for working with filenames and directories.

## Installation

Add the following to your `Cargo.toml` to use Papercut as a library:

```toml
[dependencies]
papercut = "0.1.0"
```

## Usage

### Slice an Image

```rust
use papercut::slice;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tiles = slice("image.png", Some(4), None, None, false)?;
    println!("Generated {} tiles.", tiles.len());
    Ok(())
}
```

### Save Tiles

```rust
use papercut::{save_tiles, get_basename};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tiles = slice("image.png", Some(4), None, None, false)?;
    save_tiles(&mut tiles, &get_basename("image.png"), Some(std::path::Path::new("./output")), "png")?;
    println!("Tiles saved successfully!");
    Ok(())
}
```

### Join Tiles

```rust
use papercut::join;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tiles = slice("image.png", Some(4), None, None, false)?;
    let combined_image = join(&tiles, 0, 0)?;
    combined_image.save("combined_image.png")?;
    println!("Image combined successfully!");
    Ok(())
}
```

## CLI Tool
Papercut includes a CLI tool for slicing images. To use it, build the binary and run it:

```
cargo run --bin slice-image -- --image image.png --num-tiles 4 --dir ./output --format png
```

Arguments
**--image**: Path to the image file to slice (required).
**--num-tiles**: Number of tiles to create (optional).
**--dir**: Output directory for the tiles (default: ./).
**--format**: Output image format (default: png).
**--rows**: Number of rows to divide the image (optional, used when --num-tiles is not specified).
**--columns**: Number of columns to divide the image (optional, used when --num-tiles is not specified).

Examples

Slice an Image into 4 Tiles
```shell
cargo run --bin slice-image -- --image image.png --num-tiles 4 --dir ./output --format png
```

Slice an Image into a Custom Grid
```shell
cargo run --bin slice-image -- --image image.png --rows 2 --columns 2 --dir ./output --format png
```

Combine Tiles Back into a Single Image

```rust
use papercut::{open_images_in, join};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tiles = open_images_in(Path::new("./output"))?;
    let combined_image = join(&tiles, 0, 0)?;
    combined_image.save("combined_image.png")?;
    println!("Image combined successfully!");
    Ok(())
}
```


## License
This project is licensed under the MIT License. See the LICENSE file for details.
