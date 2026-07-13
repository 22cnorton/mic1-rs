# Mic1 RS

This is a Mic1 emulator written in Rust, inspired by the [Mic1 emulator](https://cs.uml.edu/~bill/cs305/Mic1Src/) I used at UMass Lowell.

This is still a work in progress, and I want to add a GUI to make this a fully featured environment for testing Mic1 code & microcode. 

This repo includes the default microcode (prom.dat), and a sample program with source code (ifib.{asm,bin}). 

## Running

To run the emulator with the sample provided use this command:
```sh
    cargo run -- --prom prom.dat --program ifib.bin
```