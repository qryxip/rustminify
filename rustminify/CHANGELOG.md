# [0.2.0] - 2023-08-06Z

## Changed

- Adapted [the new reserving syntax in Rust 2021](https://doc.rust-lang.org/edition-guide/rust-2021/reserving-syntax.html) ([#7](https://github.com/qryxip/rustminify/pull/7) by [@mizar](https://github.com/mizar)).


    ```rust
    fn x(a:&'a u8[])->impl 'a+Clone{}
    ```

    ```rust
    's:loop{loop{break 's;}}
    ```
