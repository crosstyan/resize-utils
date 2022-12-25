import multiprocessing
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
import shutil
import dill
from multiprocessing import Process

# https://stackoverflow.com/questions/67957266/python-tqdm-process-map-append-list-shared-between-processes
# https://tqdm.github.io/docs/contrib.concurrent/
from tqdm.contrib.concurrent import process_map
from concurrent.futures import ProcessPoolExecutor

def get_relate(p: Path, root: Path, new_root: Path):
    rp = p.relative_to(root)
    return new_root.joinpath(rp)

def get_parser():
  parser = argparse.ArgumentParser(description='Resize images in a folder')
  parser.add_argument('-i', '--src', type=str, help='The source directory', required=True)
  parser.add_argument('-o', '--dst', type=str,
                      help='The destination directory', required=True)
  parser.add_argument('-s', '--size', type=int,
                      help='The size you want to fuck', default=512)
  return parser

# https://stackoverflow.com/questions/4568580/python-glob-multiple-filetypes
extensions = ("jpg", "png", "jpeg", "bmp", "webp")

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
def handle_pic(pic_path:Path, root:Path, new_root:Path, expected_l:int):
  pic = pic_path
  save_path = get_relate(pic, root, new_root)
  # skip
  # I assume it already be handled
  if save_path.with_suffix('.jpg').exists():
    return
  with Image(filename=pic) as img:
    # if you only want to remove the alpha channel and set the background to white
    if not save_path.parent.exists():
      save_path.parent.mkdir(parents=True)
    txt = pic.with_suffix(".txt")
    if txt.exists():
      dst_txt = get_relate(txt, root, new_root)
      shutil.copy(txt, dst_txt)
    if check_is_need_modify(img, expected_l):
      img.background_color = Color('white')
      img.alpha_channel = 'remove'
      img.format = 'jpg'
      # resize while preserve aspect ratio
      w, h = get_new_size(img.size, expected_l)
      img.resize(w, h, filter='lanczos')
      img.compression_quality = 99
      img.save(filename=save_path.with_suffix('.jpg'))
      # tqdm.write("Processd {}".format(pic))
      # if pic.suffix != '.jpg':
      #   os.remove(pic)
    else:
      img.save(filename=save_path.with_suffix('.jpg'))
      # print("Skip {}".format(pic))

def main():
  # https://stackoverflow.com/questions/72766345/attributeerror-cant-pickle-local-object-in-multiprocessing
  # https://stackoverflow.com/questions/46021456/configure-multiprocessing-in-python-to-use-forkserver
  # multiprocessing.set_start_method("fork", force=True) 
  # https://deecode.net/?p=975
  # https://www.pythonsheets.com/notes/python-concurrency.html
  # pwd = Path(os.path.dirname(os.path.realpath(__file__))) 
  parser = get_parser()
  args = parser.parse_args()
  src = Path(args.src) 
  dst = Path(args.dst)

  grabbed = []
  for ext in extensions:
    grabbed.extend(src.rglob("*.{}".format(ext)))

  def _handle_pic(pic_path:str):
    handle_pic(pic_path, src, dst, args.size)

  print("Process {}".format(src))
  print("Image count {}".format(len(grabbed)))
  u_count = cpu_count()
  # solve that shit later
  # process_map(_handle_pic, grabbed, max_workers=u_count)
  # TODO: handle ctrl + C
  for p in tqdm(grabbed):
    try:
      handle_pic(p, src, dst, args.size)
    except:
      tqdm.write("fucked {}".format(p))

if __name__ == "__main__":
  main()
