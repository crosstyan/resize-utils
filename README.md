# Resize utils

A dumb little library for resizing images.

Depends on [ImageMagick7](http://www.imagemagick.org/).

```bash
Usage: resize-rs [OPTIONS] --input-dir <INPUT_DIR>

Options:
  -i, --input-dir <INPUT_DIR>  The directory to search for pictures
  -q, --quality <QUALITY>      JPEG compression quality [default: 90]
  -l, --length <LENGTH>        The length you desire [default: 768]
      --no-preserve-long-side  flag to indicate whether preserve the long side. for example, for images with width > height, if this flag is set i.e. not preserve the long side, the width will be the exact length you set
  -h, --help                   Print help information
  -V, --version                Print version information
```

## TODO

- [ ] choose other format
- [ ] give the option to set the directory to save the resized images instead of overwriting the original ones (inplace for now)
