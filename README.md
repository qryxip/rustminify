# rustminify

[![CI](https://github.com/qryxip/rustminify/workflows/CI/badge.svg)](https://github.com/qryxip/rustminify/actions?workflow=CI)
[![codecov](https://codecov.io/gh/qryxip/rustminify/branch/master/graph/badge.svg)](https://codecov.io/gh/qryxip/rustminify/branch/master)
[![dependency status](https://deps.rs/repo/github/qryxip/rustminify/status.svg)](https://deps.rs/repo/github/qryxip/rustminify)

A tool for minifying Rust code.

## [`rustminify-cli`](https://crates.io/crates/rustminify-cli) (Binary)

[![Crates.io](https://img.shields.io/crates/v/rustminify-cli.svg)](https://crates.io/crates/rustminify-cli)
[![Crates.io](https://img.shields.io/crates/l/rustminify-cli.svg)](https://crates.io/crates/rustminify-cli)

```console
❯ cargo install rustminify-cli
```

```console
❯ rustminify --remove-docs <<EOF
//! crate-level doc

fn main() {
    println!("{}", module::f());
}

mod module {
    //! module-level doc

    /// doc for an item
    pub(crate) fn f() -> i32 {
        1 + 1
    }
}
EOF
fn main(){println!("{}",module::f());}mod module{pub(crate)fn f()->i32{1+1}}
```

## [`rustminify`](https://crates.io/crates/rustminify) (Library)

[![Crates.io](https://img.shields.io/crates/v/rustminify.svg)](https://crates.io/crates/rustminify)
[![Crates.io](https://img.shields.io/crates/l/rustminify.svg)](https://crates.io/crates/rustminify)

```console
❯ cargo add rustminify
```

```rust
use syn::parse_quote;

assert_eq!(
    r#"fn main(){println!("{}",module::f());}mod module{pub(crate)fn f()->i32{1+1}}"#,
    rustminify::minify_file(&rustminify::remove_docs(parse_quote! {
        //! crate-level doc

        fn main() {
            println!("{}", module::f());
        }

        mod module {
            //! module-level doc

            /// doc for an item
            pub(crate) fn f() -> i32 {
                1 + 1
            }
        }
    })),
);
```

## License

Dual-licensed under [MIT](https://opensource.org/licenses/MIT) or [Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0).
