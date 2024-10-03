# resviewer_rust

this rust project is designed to read and parse custom ilff (image) files used in *project i.g.i* and *i.g.i 2: covert strike*, extract information from them, and display their contents using a graphical user interface it uses the `eframe` and `egui` libraries for the interface, and handles binary file reading using the `byteorder` crate 
the ilff texture (`.tex`) files are stored inside `.res` files

![screenshot](https://i.imgur.com/vN69a0O.png)

## features

- **custom binary file parsing**: reads and decodes ilff files with defined structures
- **user interface with `eframe` and `egui`**: displays the extracted data visually using `egui`
- **file dialog for easy selection**: use the `rfd` library to select files interactively
- **flexible file handling**: parse different types of resource chunks, including `name` and `body` sections
- **displays image resources**: decodes and renders image resources with specified width and height

## project structure

- `src/main.rs`: contains the main logic for reading and parsing ilff files, as well as rendering the gui
- `src/fonts/inter-regular.ttf`: custom google font used for the gui interface

## installation and usage

### prerequisites

- [rust](https://www.rust-lang.org/tools/install)
- cargo, which is included in the rust installation

### building the project

1. clone the repository

    ```bash
    git clone https://github.com/remivoire/resviewer_rust.git
    ```

2. navigate to the project directory

    ```bash
    cd resviewer_rust
    ```

3. build and run the project

    ```bash
    cargo run
    ```

### using the application

1. run the application using `cargo run`
2. use the file dialog to select a `.res` file containing `.tex` textures
3. the application will parse the file and display the content, including image resources if present

## dependencies

this project uses the following rust crates

- [`eframe`](https://docs.rs/eframe/latest/eframe/): used for creating the gui interface
- [`egui`](https://docs.rs/egui/latest/egui/): gui toolkit for rust
- [`byteorder`](https://docs.rs/byteorder/latest/byteorder/): for handling binary data with ease
- [`rfd`](https://docs.rs/rfd/latest/rfd/): for displaying file dialogs

## known issues
- fails to load some textures in the `.res` file due to byte order mismatch, actively being worked upon
