# User guides [![Build Status](https://travis-ci.org/actix/book.svg?branch=master)](https://travis-ci.org/actix/book) [![Join the chat at https://gitter.im/actix/actix](https://badges.gitter.im/actix/actix.svg)](https://gitter.im/actix/actix?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

- [Actix User Guide](https://actix.rs/book/actix/)
- [Actix API Documentation (Development)](https://actix.rs/actix/actix/)
- [Actix API Documentation (Releases)](https://docs.rs/actix/)

- [Actix Web User Guide](https://actix.rs/docs/)
- [Actix Web API Documentation (Development)](https://actix.rs/actix-web/actix_web/)
- [Actix Web API Documentation (Releases)](https://docs.rs/actix-web/)

- [Chat on gitter](https://gitter.im/actix/actix)

## Using this library

This repository is an [`mdBook`](https://github.com/rust-lang-nursery/mdBook)
project. To use it for this project:

- Install `mdBook` if you haven't already: `cargo install mdbook`
- In the `actix` directory: `mdbook watch -o`
  - This automatically opens your browser and watches the md files for changing
  - You'll still have to refresh the page, as there is no hot-reloading for
    `mdbook`
