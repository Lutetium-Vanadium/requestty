extern crate proc_macro;
use proc_macro::TokenStream;

#[rustversion::since(1.51)]
macro_rules! iter {
    ($span:expr => $($tt:tt)*) => {
        quote! { ::std::array::IntoIter::new([ $($tt)* ]) }
    };
}
#[rustversion::before(1.51)]
macro_rules! iter {
    ($span:expr => $($tt:tt)*) => {
        quote! { ::std::vec![ $($tt)* ] }
    };
}

mod helpers;
mod question;

use question::*;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Token};

#[proc_macro]
pub fn questions(item: TokenStream) -> TokenStream {
    let p = parse_macro_input!(item as Questions);

    let questions = p.questions.into_iter();

    let ts = iter! { proc_macro2::Span::call_site() => #(#questions),* };

    ts.into()
}

struct Questions {
    questions: syn::punctuated::Punctuated<Question, Token![,]>,
}

impl Parse for Questions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            questions: input.parse_terminated(Question::parse)?,
        })
    }
}
