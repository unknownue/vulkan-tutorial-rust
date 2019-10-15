#!/usr/bin/env python3

# This script is modified from https://github.com/SaschaWillems/Vulkan/blob/master/download_assets.py

import sys
import os
from urllib.request import urlretrieve
from zipfile import ZipFile

ASSET_GENERAL_TEXTURE_URL = "https://vulkan-tutorial.com/images/texture.jpg"
ASSET_CHALET_TEXTURE_URL = "https://vulkan-tutorial.com/resources/chalet.jpg"
ASSET_CHALET_OBJ_URL = "https://vulkan-tutorial.com/resources/chalet.obj.zip"

ASSET_GENERAL_TEXTURE_PATH = "./assets/texture.jpg"
ASSET_CHALET_TEXTURE_PATH  = "./assets/chalet.jpg"
ASSET_CHALET_OBJ_ZIP_PATH  = "./assets/chalet.obj.zip"

def reporthook(blocknum, blocksize, totalsize):
    bytesread = blocknum * blocksize
    if totalsize > 0:
        percent = bytesread * 1e2 / totalsize
        s = "\r%5.1f%% (%*d / %d bytes)" % (percent, len(str(totalsize)), bytesread, totalsize)
        sys.stderr.write(s)
        if bytesread >= totalsize:
            sys.stderr.write("\n")
    else:
        sys.stderr.write("read %d\n" % (bytesread,))

print("Downloading CC0 licensed image...")
urlretrieve(ASSET_GENERAL_TEXTURE_URL, ASSET_GENERAL_TEXTURE_PATH, reporthook)

print("Downloading chalet texture...")
urlretrieve(ASSET_CHALET_TEXTURE_URL, ASSET_CHALET_TEXTURE_PATH, reporthook)

print("Downloading chalet obj...")
urlretrieve(ASSET_CHALET_OBJ_URL, ASSET_CHALET_OBJ_ZIP_PATH, reporthook)

print("Download finished")

print("Extracting chalet obj...")

zip = ZipFile(ASSET_CHALET_OBJ_ZIP_PATH, 'r')
zip.extractall("./assets/")
zip.close()
os.remove(ASSET_CHALET_OBJ_ZIP_PATH)

print('..done!')
