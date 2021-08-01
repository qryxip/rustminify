//! Minifies Rust code.
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]

use std::mem;

use proc_macro2::{Delimiter, LineColumn, Spacing, TokenStream, TokenTree};
use quote::{quote, ToTokens as _};
use syn::{
    parse_quote,
    visit_mut::{self, VisitMut},
    AttrStyle, Attribute, Field, FieldValue, File, ForeignItemFn, ForeignItemMacro,
    ForeignItemStatic, ForeignItemType, ImplItemConst, ImplItemMacro, ImplItemMethod, ImplItemType,
    ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMacro2,
    ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Meta,
    MetaList, NestedMeta, Path, TraitItemConst, TraitItemMacro, TraitItemMethod, TraitItemType,
    Variant,
};

/// Minifies a [`File`].
///
/// Currently this is just a shorthand for `minify_tokens(file.to_token_stream())`.
/// Unnecessary spaces may be left.
///
/// ```
/// use syn::parse_quote;
///
/// assert_eq!(
///     r#"mod a{fn hello()->&'static str{"Hello"}}"#,
///     rustminify::minify_file(&parse_quote! {
///         mod a {
///             fn hello() -> &'static str {
///                 "Hello"
///             }
///         }
///     }),
/// );
/// ```
///
/// [`File`]: https://docs.rs/syn/1/syn/struct.File.html
pub fn minify_file(file: &File) -> String {
    minify_tokens(file.to_token_stream())
}

/// Minifies a [`TokenStream`].
///
/// ```
/// use quote::quote;
///
/// assert_eq!(
///     "'a'=>1,'b'=>2,",
///     rustminify::minify_tokens(quote! {
///         'a' => 1,
///         'b' => 2,
///     }),
/// );
/// ```
///
/// [`TokenStream`]: https://docs.rs/proc-macro2/1/proc_macro2/struct.TokenStream.html
pub fn minify_tokens(tokens: TokenStream) -> String {
    let safe = tokens.to_string();
    let mut acc = "".to_owned();
    minify_tokens(tokens.clone(), &mut acc);
    return if acc.parse().map_or(false, |acc| equiv(acc, tokens)) {
        acc
    } else {
        safe
    };

    fn minify_tokens(tokens: TokenStream, acc: &mut String) {
        let mut st = State::None;
        for tt in tokens {
            match tt {
                TokenTree::Group(group) => {
                    if let State::PunctChars(puncts, _, _) = mem::replace(&mut st, State::None) {
                        *acc += &puncts;
                    }
                    let (left, right) = match group.delimiter() {
                        proc_macro2::Delimiter::Parenthesis => ('(', ')'),
                        proc_macro2::Delimiter::Brace => ('{', '}'),
                        proc_macro2::Delimiter::Bracket => ('[', ']'),
                        proc_macro2::Delimiter::None => (' ', ' '),
                    };
                    acc.push(left);
                    minify_tokens(group.stream(), acc);
                    acc.push(right);
                    st = State::None;
                }
                TokenTree::Ident(ident) => {
                    match mem::replace(&mut st, State::AlnumUnderscoreQuote) {
                        State::AlnumUnderscoreQuote => *acc += " ",
                        State::PunctChars(puncts, _, _) => *acc += &puncts,
                        _ => {}
                    }
                    *acc += &ident.to_string();
                }
                TokenTree::Literal(literal) => {
                    let end = literal.span().end();
                    let literal = literal.to_string();
                    let (literal, next) = if let Some(literal) = literal.strip_suffix('.') {
                        (
                            literal,
                            State::PunctChars(".".to_owned(), end, Spacing::Alone),
                        )
                    } else {
                        (&*literal, State::AlnumUnderscoreQuote)
                    };
                    match mem::replace(&mut st, next) {
                        State::AlnumUnderscoreQuote => *acc += " ",
                        State::PunctChars(puncts, _, _) => *acc += &puncts,
                        _ => {}
                    }
                    *acc += &literal.to_string();
                }
                TokenTree::Punct(punct) => {
                    let cur_pos = punct.span().start();
                    if let State::PunctChars(puncts, prev_pos, spacing) = &mut st {
                        if *spacing == Spacing::Alone {
                            *acc += puncts;
                            // https://docs.rs/syn/1.0.46/syn/token/index.html
                            if !adjacent(*prev_pos, cur_pos)
                                && [
                                    ("!", '='),
                                    ("%", '='),
                                    ("&", '&'),
                                    ("&", '='),
                                    ("*", '='),
                                    ("+", '='),
                                    ("-", '='),
                                    ("-", '>'),
                                    (".", '.'),
                                    ("..", '.'),
                                    ("..", '='),
                                    ("/", '='),
                                    (":", ':'),
                                    ("<", '-'),
                                    ("<", '<'),
                                    ("<", '='),
                                    ("<<", '='),
                                    ("=", '='),
                                    ("=", '>'),
                                    (">", '='),
                                    (">", '>'),
                                    (">>", '='),
                                    ("^", '='),
                                    ("|", '='),
                                    ("|", '|'),
                                ]
                                .contains(&(puncts, punct.as_char()))
                            {
                                *acc += " ";
                            }
                            st = State::PunctChars(
                                punct.as_char().to_string(),
                                cur_pos,
                                punct.spacing(),
                            );
                        } else {
                            puncts.push(punct.as_char());
                            *spacing = punct.spacing();
                        }
                    } else {
                        st = State::PunctChars(
                            punct.as_char().to_string(),
                            cur_pos,
                            punct.spacing(),
                        );
                    }
                }
            }
        }
        if let State::PunctChars(puncts, _, _) = st {
            *acc += &puncts;
        }

        fn adjacent(pos1: LineColumn, pos2: LineColumn) -> bool {
            pos1.line == pos2.line && pos1.column + 1 == pos2.column
        }

        #[derive(PartialEq)]
        enum State {
            None,
            AlnumUnderscoreQuote,
            PunctChars(String, LineColumn, Spacing),
        }
    }

    fn equiv(tokens1: TokenStream, tokens2: TokenStream) -> bool {
        return compress(tokens1) == compress(tokens2);

        fn compress(tokens: TokenStream) -> Vec<LossyTokenTree> {
            tokens.into_iter().map(Into::into).collect()
        }

        #[derive(PartialEq)]
        enum LossyTokenTree {
            Group(Delimiter, Vec<Self>),
            Ident(String),
            Punct(char),
            Literal(String),
        }

        impl From<TokenTree> for LossyTokenTree {
            fn from(tt: TokenTree) -> Self {
                match tt {
                    TokenTree::Group(group) => Self::Group(
                        group.delimiter(),
                        group.stream().into_iter().map(Into::into).collect(),
                    ),
                    TokenTree::Ident(ident) => Self::Ident(ident.to_string()),
                    TokenTree::Punct(punct) => Self::Punct(punct.as_char()),
                    TokenTree::Literal(literal) => Self::Literal(literal.to_string()),
                }
            }
        }
    }
}

/// Removes doc comments and doc attributes.
///
/// Also removes `#[{warn, deny, forbid}(missing_docs, missing_crate_level_docs)]`.
///
/// ```
/// use syn::parse_quote;
///
/// assert_eq!(
///     "#![deny()]pub mod module{pub struct Item;}",
///     rustminify::minify_file(&rustminify::remove_docs(parse_quote! {
///         #![deny(missing_docs)]
///         pub mod module {
///             //! module
///
///             /// Item
///             pub struct Item;
///         }
///     }))
/// );
/// ```
pub fn remove_docs(mut file: File) -> File {
    Visitor.visit_file_mut(&mut file);
    return file;

    struct Visitor;

    macro_rules! visit(($($f:ident(_, &mut $ty:ty);)*) => {
        $(
            fn $f(&mut self, node: &mut $ty) {
                remove_docs(&mut node.attrs);
                visit_mut::$f(self, node);
            }
        )*
    });

    impl VisitMut for Visitor {
        visit! {
            visit_field_mut(_, &mut Field);
            visit_field_value_mut(_, &mut FieldValue);
            visit_file_mut(_, &mut File);
            visit_foreign_item_fn_mut(_, &mut ForeignItemFn);
            visit_foreign_item_macro_mut(_, &mut ForeignItemMacro);
            visit_foreign_item_static_mut(_, &mut ForeignItemStatic);
            visit_foreign_item_type_mut(_, &mut ForeignItemType);
            visit_impl_item_const_mut(_, &mut ImplItemConst);
            visit_impl_item_macro_mut(_, &mut ImplItemMacro);
            visit_impl_item_method_mut(_, &mut ImplItemMethod);
            visit_impl_item_type_mut(_, &mut ImplItemType);
            visit_item_const_mut(_, &mut ItemConst);
            visit_item_enum_mut(_, &mut ItemEnum);
            visit_item_extern_crate_mut(_, &mut ItemExternCrate);
            visit_item_fn_mut(_, &mut ItemFn);
            visit_item_foreign_mod_mut(_, &mut ItemForeignMod);
            visit_item_impl_mut(_, &mut ItemImpl);
            visit_item_macro2_mut(_, &mut ItemMacro2);
            visit_item_macro_mut(_, &mut ItemMacro);
            visit_item_mod_mut(_, &mut ItemMod);
            visit_item_static_mut(_, &mut ItemStatic);
            visit_item_struct_mut(_, &mut ItemStruct);
            visit_item_trait_alias_mut(_, &mut ItemTraitAlias);
            visit_item_trait_mut(_, &mut ItemTrait);
            visit_item_type_mut(_, &mut ItemType);
            visit_item_union_mut(_, &mut ItemUnion);
            visit_item_use_mut(_, &mut ItemUse);
            visit_trait_item_const_mut(_, &mut TraitItemConst);
            visit_trait_item_macro_mut(_, &mut TraitItemMacro);
            visit_trait_item_method_mut(_, &mut TraitItemMethod);
            visit_trait_item_type_mut(_, &mut TraitItemType);
            visit_variant_mut(_, &mut Variant);
        }
    }

    fn remove_docs(attrs: &mut Vec<Attribute>) {
        attrs.retain(|a| !matches!(a.parse_meta(), Ok(m) if m.path().is_ident("doc")));
        for attr in attrs {
            if let Ok(Meta::List(MetaList { path, nested, .. })) = attr.parse_meta() {
                if any(&path, &["warn", "deny", "forbid"]) {
                    let bang = matches!(attr.style, AttrStyle::Inner(_)).then(|| quote!(!));
                    let nested = nested.into_iter().filter(|nested| {
                        !matches!(
                            nested, NestedMeta::Meta(Meta::Path(path))
                            if any(path, &["missing_docs", "missing_crate_level_docs"])
                        )
                    });
                    *attr = parse_quote!(##bang[#path(#(#nested),*)]);
                }
            }
        }

        fn any(path: &Path, idents: &[&str]) -> bool {
            idents.iter().any(|s| path.is_ident(s))
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens as _};
    use syn::{parse_quote, File};
    use test_case::test_case;

    #[test_case(quote!(a + *b)                                => "a+*b"                           ; "joint_add_deref"       )]
    #[test_case(quote!(a + !b)                                => "a+!b"                           ; "joint_add_not"         )]
    #[test_case(quote!(a + -b)                                => "a+-b"                           ; "joint_add_neg"         )]
    #[test_case(quote!(a + &b)                                => "a+&b"                           ; "joint_add_reference"   )]
    #[test_case(quote!(a && &b)                               => "a&&&b"                          ; "joint_andand_reference")]
    #[test_case(quote!(a & &b)                                => "a& &b"                          ; "space_and_reference"   )]
    #[test_case(quote!(a < -b)                                => "a< -b"                          ; "space_le_neg"          )]
    #[test_case(quote!(0. ..1.)                               => "0. ..1."                        ; "space_dec_point_range" )]
    #[test_case(quote!(x | || ())                             => "x| ||()"                        ; "zero_arg_closure"      )]
    #[test_case(quote!(println!("{}", 2 * 2 + 1))             => r#"println!("{}",2*2+1)"#        ; "println"               )]
    #[test_case(quote!(macro_rules! m { ($($_:tt)*) => {}; }) => "macro_rules!m{($($_:tt)*)=>{};}"; "macro_rules"           )]
    fn minify_tokens(tokens: TokenStream) -> String {
        crate::minify_tokens(tokens)
    }

    #[test_case(parse_quote!(#[doc = ""] pub struct Item;) => "pub struct Item;"; "item_struct"      )]
    #[test_case(parse_quote!(#![deny(missing_docs)])       => "#![deny()]"      ; "deny_missing_docs")]
    fn minify_tokens_with_remove_docs(file: File) -> String {
        crate::minify_tokens(crate::remove_docs(file).into_token_stream())
    }
}
