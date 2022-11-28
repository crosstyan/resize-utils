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
parser.add_argument('-p','--path', type=str, default='.', help='path to the folder containing images')

# https://stackoverflow.com/questions/4568580/python-glob-multiple-filetypes
extensions = ("jpg", "png", "gif", "jpeg", "bmp", "webp")

def check_is_need_modify(img:Image, expected_l:int):
  pic_format = img.format.lower()
  format_criteria = pic_format == 'jpeg' or pic_format == "jpg"
  # print("format_criteria: {}, {}".format(format_criteria, pic_format))
  w, h = img.size
  size_criteria: bool = (w > expected_l) and (h > expected_l)
  # modify image if it's not jpeg or it's size is larger than expected_l
  # print("size_criteria: {}, {}, {}".format(size_criteria, w, h))
  return (not format_criteria) or size_criteria

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
def handle_pic(pic_path:str, expected_l:int, ng_path:str):
  pic = Path(pic_path)
  if pic.suffix == ".gif":
    try:
      if ng_path is not None:
        dump_path = ng_path.joinpath(pic.name)
        tqdm.write("Skipping gif file {}. Dump it to somewhere else.".format(pic))
        shutil.move(pic, dump_path)
    except:
      print("Failed to move gif file to {}".format(dump_path))
      return None
  with Image(filename=pic) as img:
    # if you only want to remove the alpha channel and set the background to white
    if check_is_need_modify(img, expected_l):
      img.background_color = Color('white')
      img.alpha_channel = 'remove'
      img.format = 'jpg'
      # resize while preserve aspect ratio
      w, h = get_new_size(img.size, expected_l)
      img.resize(w, h, filter='lanczos')
      img.compression_quality = 90
      img.save(filename=pic.with_suffix('.jpg'))
      # tqdm.write("Processd {}".format(pic))
      if pic.suffix != '.jpg':
        os.remove(pic)
    else:
      pass
      # print("Skip {}".format(pic))

global ng_dump
ng_dump = None
def _handle_pic(pic_path:str):
  handle_pic(pic_path, 768, ng_dump)

if __name__ == "__main__":
  # https://www.pythonsheets.com/notes/python-concurrency.html
  # pwd = Path(os.path.dirname(os.path.realpath(__file__))) 
  args = parser.parse_args()
  base_dir = Path(args.path)
  base_dir_recursive = base_dir.joinpath("./**")
  ng_dump = base_dir.joinpath("..", "ng")

  if not ng_dump.exists():
    # create folder
    # print("Creating no good dump folder at {}".format(ng_dump))
    ng_dump.mkdir(parents=True, exist_ok=True)
    # print("skip dump")
    # ng_dump = None
    print("make dump {}".format(ng_dump))

  grabbed = []
  for ext in extensions:
    grabbed.extend(glob.glob(os.path.join(base_dir_recursive, f"*.{ext}"), recursive=True))

  print("Process {}".format(base_dir))
  print("Image count {}".format(len(grabbed)))
  u_count = cpu_count()
  process_map(_handle_pic, grabbed, max_workers=u_count)
