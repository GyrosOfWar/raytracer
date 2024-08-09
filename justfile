set shell := ["nu", "-c"]

project := justfile_directory()

prepare:
    bzip2 -d data/color-spaces/*.bz2