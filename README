# sisa-assembler
This is an assembler for the SISA assembly language, written in Rust!

## Motivation
 - Challenge myself to develop an assembler
 - Provide a tool that I missed when I took the IC subject to current students

## Installation
Execute the following commands:
```shell
git clone https://github.com/rdvdev2/sisa-assembler.git
cd sisa-assembler
cargo install --path .
```

If you haven't done it yet, add `.cargo/bin` to your `$PATH`
```shell
# At the end of your .bashrc / .zshrc
export PATH=$PATH:$HOME/.cargo/bin
```

## Usage
Run `sas -h` to see the program help:
```shell
The SISA assembler by rdvdev2<me@rdvdev2.com>

Usage: sas [OPTIONS]

Recognized options:
    -i, --input FILE                Uses FILE as input (source.S by default)
    -o, --output FILE               Uses FILE as output (out.bin by default)

    --text-section-start ADDRESS    Places the .text section in ADDRESS (0x0000 by default)
    --data-section-start ADDRESS    Places the .data section in ADDRESS (right after .text by default)
    --auto-align-words              Automatically aligns words to multiples of 2 (disabled by default)
    --auto-align-sections           Automatically aligns sections to multiples of 2 (disabled by default)

    -h, --help                      Shows this help message
```

## The language
The language (as well as the ISA) is defined by the documentation of the IC subject of the Computer Engineering course
of the UPC. I won't include the specification here as I'm not sure about its licensing. I wrote the assembler following
this specification as close as possible, but be aware that this is a personal project and as such the implementation may
not be perfect. Report any issues you find!

## Notes for current IC students
Here are some notes for IC students that may use this assembler in their study:
 - The assembler flags `--auto-align-words` and `--auto-align-words` aren't part of the official specification, use
   `.even` instead
 - The assembler puts the `.data` section immediately after the `.text` section by default. Ensure that this is the
   desired behaviour before assembling. If it isn't, check the program help to relocate the sections.
 - Literals are always interpreted as signed twos-compliment values. This means that you can write `.byte 0xFFFF` and
   the assembler will interpret it as `.byte -1`, effectively translating a word into a byte. This is possibly not
   desirable when writing programs for your assignments, and you should avoid taking advantage of this feature.

## Project status
This is a (somewhat loose) roadmap of the project. Take it with a grain of salt, I may not implement everything in the
list!
- [x] Specification compliance
- [x] Section relocation
- [x] Error detection
- [ ] Warnings
  - [x] Instructions in .data
  - [x] Raw data in .text
  - [ ] Unused symbols
  - [ ] Unaligned data
  - [ ] Unaligned instruction
- [ ] Help messages
  - [ ] Use lo() and hi() to load words
- [ ] Indicate span of non-syntactic errors
- [ ] `--enforce-ic-compliance` flag
- [ ] Tests