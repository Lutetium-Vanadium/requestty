extern crate proc_macro;
use proc_macro::TokenStream;

#[rustversion::since(1.51)]
macro_rules! iter {
    ($span:expr => $($tt:tt)*) => {
        quote::quote_spanned! { $span => ::std::array::IntoIter::new([ $($tt)* ]) }
    };
}
#[rustversion::before(1.51)]
macro_rules! iter {
    ($span:expr => $($tt:tt)*) => {
        quote::quote_spanned! { $span => ::std::vec![ $($tt)* ] }
    };
}

mod helpers;
mod question;

use question::*;
use syn::{parse::Parse, parse_macro_input, Token};

#[proc_macro]
pub fn questions(item: TokenStream) -> TokenStream {
    let p = parse_macro_input!(item as Questions);

    let questions = p.questions.into_iter();

    if p.inline {
        iter! { proc_macro2::Span::call_site() => #(#questions),* }
    } else {
        quote::quote! { ::std::vec![ #(#questions),* ] }
    }
    .into()
}

struct Questions {
    inline: bool,
    questions: syn::punctuated::Punctuated<Question, Token![,]>,
}

impl Parse for Questions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inline = input
            .step(|c| match c.ident() {
                Some((ident, c)) if ident == "inline" => Ok(((), c)),
                _ => Err(c.error("no inline")),
            })
            .is_ok();

        Ok(Self {
            inline,
            questions: input.parse_terminated(Question::parse)?,
        })
    }
}
