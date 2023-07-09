# TempleOS Risen 
For practicing Rust.

## Usage
To run the program, use the following command
```bash
# cargo run <output> <BPM>
cargo run speaker simple 90 
# or
cargo run file faded 120
# or the templeos way
cargo run speaker templeos 120
```
Note that templeos with file option is not working due to 32-bit wav file format not supported by synthrs.