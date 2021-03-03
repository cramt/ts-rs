use std::convert::TryFrom;

use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::ToTokens;
use regex::Regex;
use std::ops::Deref;
use syn::{Attribute, Type};
use syn::{Error, Result};

macro_rules! syn_err {
    ($l:literal $(, $a:expr)*) => {
        syn_err!(proc_macro2::Span::call_site(); $l $(, $a)*);
    };
    ($s:expr; $l:literal $(, $a:expr)*) => {
        return Err(syn::Error::new($s, format!($l $(, $a)*)));
    };
}

macro_rules! impl_parse {
    ($i:ident ($input:ident, $out:ident) { $($k:pat => $e:expr),* $(,)? }) => {
        impl std::convert::TryFrom<&syn::Attribute> for $i {
            type Error = syn::Error;

            fn try_from(attr: &syn::Attribute) -> syn::Result<Self> { attr.parse_args() }
        }

        impl syn::parse::Parse for $i {
            fn parse($input: syn::parse::ParseStream) -> syn::Result<Self> {
                let mut $out = $i::default();
                loop {
                    let key: Ident = $input.call(syn::ext::IdentExt::parse_any)?;
                    match &*key.to_string() {
                        $($k => $e,)*
                        #[allow(unreachable_patterns)]
                        _ => syn_err!($input.span(); "unexpected attribute")
                    }

                    match $input.is_empty() {
                        true => break,
                        false => {
                            $input.parse::<syn::Token![,]>()?;
                        }
                    }
                }

                Ok($out)
            }
        }
    };
}

/// Parse all `#[ts(..)]` attributes from the given slice.
pub fn parse_attrs<'a, A>(attrs: &'a [Attribute]) -> Result<impl Iterator<Item = A>>
where
    A: TryFrom<&'a Attribute, Error = Error>,
{
    Ok(attrs
        .iter()
        .filter(|a| a.path.is_ident("ts"))
        .map(A::try_from)
        .collect::<Result<Vec<A>>>()?
        .into_iter())
}

/// Parse all `#[serde(..)]` attributes from the given slice.
#[cfg(feature = "serde-compat")]
#[allow(unused)]
pub fn parse_serde_attrs<'a, A: TryFrom<&'a Attribute, Error = Error>>(
    attrs: &'a [Attribute],
) -> impl Iterator<Item = A> {
    attrs
        .iter()
        .filter(|a| a.path.is_ident("serde"))
        .flat_map(|attr| match A::try_from(attr) {
            Ok(attr) => Some(attr),
            Err(_) => {
                use quote::ToTokens;
                warning::print_warning(
                    "failed to parse serde attribute",
                    format!("{}", attr.to_token_stream()),
                    "ts-rs failed to parse this attribute. It will be ignored.",
                )
                .unwrap();
                None
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
}

#[cfg(feature = "serde-compat")]
mod warning {
    use std::fmt::Display;
    use std::io::Write;

    use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

    // Sadly, it is impossible to raise a warning in a proc macro.
    // This function prints a message which looks like a compiler warning.
    pub fn print_warning(
        title: impl Display,
        content: impl Display,
        note: impl Display,
    ) -> std::io::Result<()> {
        let make_color = |color: Color, bold: bool| {
            let mut spec = ColorSpec::new();
            spec.set_fg(Some(color)).set_bold(bold).set_intense(true);
            spec
        };

        let yellow_bold = make_color(Color::Yellow, true);
        let white_bold = make_color(Color::White, true);
        let white = make_color(Color::White, false);
        let blue = make_color(Color::Blue, true);

        let writer = BufferWriter::stderr(ColorChoice::Auto);
        let mut buffer = writer.buffer();

        buffer.set_color(&yellow_bold)?;
        write!(&mut buffer, "warning")?;
        buffer.set_color(&white_bold)?;
        writeln!(&mut buffer, ": {}", title)?;

        buffer.set_color(&blue)?;
        writeln!(&mut buffer, "  | ")?;

        write!(&mut buffer, "  | ")?;
        buffer.set_color(&white)?;
        writeln!(&mut buffer, "{}", content)?;

        buffer.set_color(&blue)?;
        writeln!(&mut buffer, "  | ")?;

        write!(&mut buffer, "  = ")?;
        buffer.set_color(&white_bold)?;
        write!(&mut buffer, "note: ")?;
        buffer.set_color(&white)?;
        writeln!(&mut buffer, "{}", note)?;

        writer.print(&buffer)
    }
}

static LIFETIME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"'[^\s,>]+").unwrap());

pub fn convert_lifetime_to_static(ty: &Type) -> TokenStream {
    let str = ty.to_token_stream().to_string();
    LIFETIME_REGEX
        .replace(str.as_str(), "'static")
        .deref()
        .parse()
        .unwrap()
}
