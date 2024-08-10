set shell := ["nu", "-c"]

project := justfile_directory()

prepare:
    bzip2 -kd data/color-spaces/*.bz2