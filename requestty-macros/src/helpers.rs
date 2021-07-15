use quote::{quote, quote_spanned, ToTokens};
use syn::{parse::Parse, spanned::Spanned, Token};

pub(crate) fn parse_optional_comma(
    input: syn::parse::ParseStream,
) -> syn::Result<Option<Token![,]>> {
    let lookahead = input.lookahead1();
    if !lookahead.peek(Token![,]) {
        if input.is_empty() {
            return Ok(None);
        } else {
            return Err(lookahead.error());
        }
    }

    Ok(Some(input.parse::<Token![,]>()?))
}

pub(crate) fn insert_non_dup<T: Parse + From<syn::ExprPath>>(
    ident: syn::Ident,
    item: &mut Option<T>,
    input: syn::parse::ParseStream,
) -> syn::Result<()> {
    insert_non_dup_parse(ident, item, input, T::parse)
}

pub(crate) fn insert_non_dup_parse<T: From<syn::ExprPath>>(
    ident: syn::Ident,
    item: &mut Option<T>,
    input: syn::parse::ParseStream,
    parser: fn(syn::parse::ParseStream) -> syn::Result<T>,
) -> syn::Result<()> {
    check_non_dup(&ident, item)?;

    let lookahead = input.lookahead1();
    let value = if input.is_empty() || lookahead.peek(Token![,]) {
        let mut path_segments = syn::punctuated::Punctuated::new();

        path_segments.push_value(syn::PathSegment {
            ident,
            arguments: syn::PathArguments::None,
        });

        syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: path_segments,
            },
        }
        .into()
    } else if lookahead.peek(Token![:]) {
        input.parse::<Token![:]>()?;
        parser(input)?
    } else {
        return Err(lookahead.error());
    };

    *item = Some(value);

    Ok(())
}

pub(crate) fn check_non_dup<T>(ident: &syn::Ident, item: &Option<T>) -> syn::Result<()> {
    match item {
        Some(_) => Err(syn::Error::new(
            ident.span(),
            format!("duplicate option `{}`", ident),
        )),
        None => Ok(()),
    }
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum Choices {
    Array(syn::punctuated::Punctuated<Choice, Token![,]>),
    Expr(syn::Expr),
}

impl Choices {
    fn parse_impl(
        input: syn::parse::ParseStream,
        parser: fn(syn::parse::ParseStream) -> syn::Result<Choice>,
    ) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);

            Ok(Choices::Array(content.parse_terminated(parser)?))
        } else {
            Ok(Choices::Expr(input.parse()?))
        }
    }

    pub(crate) fn parse_choice(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Choices::parse_impl(input, Choice::parse)
    }

    pub(crate) fn parse_multi_select_choice(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Choices::parse_impl(input, parse_multi_select_choice)
    }
}

impl From<syn::ExprPath> for Choices {
    fn from(path: syn::ExprPath) -> Self {
        Self::Expr(path.into())
    }
}

impl ToTokens for Choices {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Choices::Array(ref elems) => {
                let mut choices = proc_macro2::TokenStream::new();
                for elem in elems.iter() {
                    elem.to_tokens(&mut choices);
                    choices.extend(quote! { , })
                }

                tokens.extend(iter! {
                    elems.span() => #choices
                });
            }
            Choices::Expr(ref choices) => {
                choices.to_tokens(tokens);
            }
        }
    }
}

pub(crate) enum Choice {
    Choice(syn::Expr),
    Separator(syn::Expr),
    DefaultSeparator,
}

impl Parse for Choice {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match input.fork().parse::<syn::Ident>() {
            Ok(i) if i == "sep" || i == "separator" => {
                input.parse::<syn::Ident>()?;
                if input.is_empty() || input.peek(Token![,]) {
                    Ok(Choice::DefaultSeparator)
                } else {
                    Ok(Choice::Separator(input.parse()?))
                }
            }
            _ => Ok(Choice::Choice(input.parse()?)),
        }
    }
}

impl ToTokens for Choice {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let (ident_name, elem) = match self {
            Choice::Choice(elem) => ("Choice", elem),
            Choice::Separator(elem) => ("Separator", elem),
            Choice::DefaultSeparator => {
                tokens.extend(quote! { ::requestty::DefaultSeparator });
                return;
            }
        };

        let ident = syn::Ident::new(ident_name, elem.span());

        tokens.extend(quote_spanned! {
            elem.span() => ::requestty::#ident(
                #[allow(clippy::useless_conversion)]
                ::std::convert::From::from(#elem)
            )
        })
    }
}

fn make_into(expr: syn::Expr) -> syn::Expr {
    let mut from_path_segments = syn::punctuated::Punctuated::new();

    from_path_segments.push(syn::PathSegment {
        ident: syn::Ident::new("std", expr.span()),
        arguments: syn::PathArguments::None,
    });
    from_path_segments.push(syn::PathSegment {
        ident: syn::Ident::new("convert", expr.span()),
        arguments: syn::PathArguments::None,
    });
    from_path_segments.push(syn::PathSegment {
        ident: syn::Ident::new("From", expr.span()),
        arguments: syn::PathArguments::None,
    });
    from_path_segments.push(syn::PathSegment {
        ident: syn::Ident::new("from", expr.span()),
        arguments: syn::PathArguments::None,
    });

    syn::ExprCall {
        attrs: Vec::new(),
        func: Box::new(
            syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: syn::Path {
                    leading_colon: Some(syn::token::Colon2(expr.span())),
                    segments: from_path_segments,
                },
            }
            .into(),
        ),
        paren_token: syn::token::Paren(expr.span()),
        args: {
            let mut args = syn::punctuated::Punctuated::new();
            args.push(expr);
            args
        },
    }
    .into()
}

// For multi_select, defaults can be given for each option, this method, takes option
// (`choice`), and the default value to put (`default`), and produces the following as
// an `Expr`:
// ```
// (choice.into(), default.into())`
// ```
fn make_multi_select_tuple(choice: syn::Expr, default: syn::Expr) -> syn::Expr {
    let paren_token = syn::token::Paren(
        choice
            .span()
            .join(default.span())
            .unwrap_or_else(|| choice.span()),
    );

    let mut elems = syn::punctuated::Punctuated::new();
    elems.push_value(make_into(choice));
    elems.push(make_into(default));

    syn::ExprTuple {
        attrs: Vec::new(),
        paren_token,
        elems,
    }
    .into()
}

fn parse_multi_select_choice(input: syn::parse::ParseStream) -> syn::Result<Choice> {
    let choice = input.parse()?;

    let choice = match choice {
        Choice::Choice(choice) if input.peek(Token![default]) => {
            input.parse::<Token![default]>()?;
            Choice::Choice(make_multi_select_tuple(choice, input.parse()?))
        }
        Choice::Choice(choice) => {
            let span = choice.span();
            Choice::Choice(make_multi_select_tuple(
                choice,
                syn::ExprLit {
                    lit: syn::LitBool { value: false, span }.into(),
                    attrs: Vec::new(),
                }
                .into(),
            ))
        }
        sep => sep,
    };

    Ok(choice)
}
