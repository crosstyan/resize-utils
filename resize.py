from pathlib import Path
import glob
import os
# from PIL import Image
from wand.image import Image
from wand.color import Color
import shutil
import argparse
from pprint import pprint
from tqdm import tqdm
from psutil import cpu_count

# https://stackoverflow.com/questions/67957266/python-tqdm-process-map-append-list-shared-between-processes
# https://tqdm.github.io/docs/contrib.concurrent/
from tqdm.contrib.concurrent import process_map
from concurrent.futures import ProcessPoolExecutor

parser = argparse.ArgumentParser(description='Resize images in a folder')
parser.add_argument('--path', type=str, default='.', help='path to the folder containing images')
parser.add_argument('--ng-path', type=str, default="", help='path to the folder containing no good images')


# https://stackoverflow.com/questions/4568580/python-glob-multiple-filetypes
extensions = ("jpg", "png", "gif", "jpeg", "bmp", "webp")

def get_new_size(old_size: tuple[int, int], new_l: int, preserve_long=True) -> tuple[int, int]:
  old_w, old_h = old_size
  # don't do anything if the image is already smaller than the new size
  if old_w < new_l or old_h < new_l:
    return old_size
  if preserve_long:
    if old_w > old_h:
      new_h = new_l
      new_w = int(new_l * old_w / old_h)
      return (new_w, new_h)
    else:
      new_w = new_l
      new_h = int(new_l * old_h / old_w)
      return (new_w, new_h)
  else:
    if old_w > old_h:
      new_w = new_l
      new_h = int(new_l * old_h / old_w)
      return (new_w, new_h)
    else:
      new_h = new_l
      new_w = int(new_l * old_w / old_h)
      return (new_w, new_h)


# https://stackoverflow.com/questions/27826854/python-wand-convert-pdf-to-png-disable-transparent-alpha-channel
def handle_pic(pic_path:str):
  pic = Path(pic_path)
  if pic.suffix == ".gif":
    dump_path = ng_dump.joinpath(pic.name)
    try:
      if ng_dump is not None:
        tqdm.write("Skipping gif file {}. Dump it to somewhere else.".format(pic))
        shutil.move(pic, dump_path)
    except:
      print("Failed to move gif file to {}".format(dump_path))
      return None
  with Image(filename=pic) as img:
    # if you only want to remove the alpha channel and set the background to white
    img.background_color = Color('white')
    img.alpha_channel = 'remove'
    img.format = 'jpg'
    # resize while preserve aspect ratio
    expected_size = 768
    w, h = get_new_size(img.size, expected_size)
    img.resize(w, h, filter='lanczos')
    img.compression_quality = 90
    img.save(filename=pic.with_suffix('.jpg'))
    tqdm.write("Processd {}".format(pic))
  if pic.suffix != '.jpg':
    os.remove(pic)


if __name__ == "__main__":
  # https://www.pythonsheets.com/notes/python-concurrency.html
  # pwd = Path(os.path.dirname(os.path.realpath(__file__))) 
  args = parser.parse_args()
  base_dir = Path(args.path)
  base_dir_recursive = base_dir.joinpath("./**")
  ng_dump = base_dir.joinpath("..", "ng")
    
  if args.ng_path != "":
    ng_dump = Path(args.ng_path)

  if not ng_dump.exists():
    # create folder
    # print("Creating no good dump folder at {}".format(ng_dump))
    # ng_dump.mkdir(parents=True, exist_ok=True)
    print("skip dump")
    ng_dump = None

  grabbed = []
  for ext in extensions:
    grabbed.extend(glob.glob(os.path.join(base_dir_recursive, f"*.{ext}"), recursive=True))

  print("Process {}".format(base_dir))
  print("Image count {}".format(len(grabbed)))
  u_count = cpu_count()
  process_map(handle_pic, grabbed, max_workers=u_count)
