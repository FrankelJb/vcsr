## Video Contact Sheets (in) Rust
A video contact sheet generator written in rust.

[![Build Status](https://travis-ci.com/FrankelJb/vcsr.svg?branch=master)](https://travis-ci.com/FrankelJb/vcsr)

### Reasons for existing
This application is a direct port of [vcsi](https://github.com/amietn/vcsi). Full credit goes to the author for the algorithm and design.

I am learning Rust and while working on another project, I required a contact sheet generator. The existing generators needed some other tools during installation and I wanted something that I could just drop in and use. The process allowed me to focus on the language's ergonomics rather than the algorithm.

### Usage
The simplest usage is to run it with no extra arguments other than an input file. See the full list of arguments below for default values.
```
$ vcsr bbbb_sunflower_1080p_60fps_normal.mp4
```
