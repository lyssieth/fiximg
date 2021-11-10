# fiximg

An image optimization and compression program for use with large libraries where filenames don't matter.

## Requirements

Please make sure [`jpegoptim`](https://github.com/tjko/jpegoptim) is installed, as currently it cannot handle it being missing.

## Usage

> ```bash
> $ fiximg
> ```

This assumes a `./data` directory where the original images are.  
It automatically creates a `./data-out` directory where the output will be.

Currently, it only knows how to optimize `.png` and `.jpeg`/`.jpg` files.  
The output file format is `BLAKE3_HASH.EXT`.

## To-do

- [ ] Detection method for `jpegoptim` (`which jpegoptim` into a shell probably, will need a windows solution too, since there is a version of jpegoptim available for Windows)
- [ ] A proper command line program, instead of expecting everything
- [ ] Various configuration settings
- [ ] Maybe a library version that can be imported into other programs
- [ ] Better console output

## Used Libraries and Software

- [rayon](https://github.com/rayon-rs/rayon) (running across multiple threads)
- [blake3](https://github.com/blake3-team/blake3) (data hashing for filenames)
- [oxipng](https://github.com/shssoichiro/oxipng) (png optimization)
- [jpegoptim](https://github.com/tjko/jpegoptim) (jpeg optimization)
