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

pub(crate) fn insert_non_dup<T: Parse>(
    ident: syn::Ident,
    item: &mut Option<T>,
    input: syn::parse::ParseStream,
) -> syn::Result<()> {
    insert_non_dup_parse(ident, item, input, T::parse)
}

pub(crate) fn insert_non_dup_parse<T>(
    ident: syn::Ident,
    item: &mut Option<T>,
    input: syn::parse::ParseStream,
    parser: fn(syn::parse::ParseStream) -> syn::Result<T>,
) -> syn::Result<()> {
    match item {
        Some(_) => Err(syn::Error::new(
            ident.span(),
            format!("duplicate option `{}`", ident),
        )),
        None => {
            *item = Some(parser(input)?);
            Ok(())
        }
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

    pub(crate) fn parse_checkbox_choice(
        input: syn::parse::ParseStream,
    ) -> syn::Result<Self> {
        Choices::parse_impl(input, parse_checkbox_choice)
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

                tokens.extend(quote_spanned! {
                    elems.span() => ::std::array::IntoIter::new([ #choices ])
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
                tokens.extend(quote! { ::inquisition::DefaultSeparator });
                return;
            }
        };

        let ident = syn::Ident::new(ident_name, elem.span());

        tokens.extend(quote_spanned! {
            elem.span() => ::inquisition::#ident(
                #[allow(clippy::useless_conversion)]
                #elem.into()
            )
        })
    }
}

fn make_into(expr: syn::Expr) -> syn::Expr {
    syn::ExprMethodCall {
        attrs: Vec::new(),
        dot_token: syn::token::Dot(expr.span()),
        method: syn::Ident::new("into", expr.span()),
        paren_token: syn::token::Paren(expr.span()),
        turbofish: None,
        args: syn::punctuated::Punctuated::new(),
        receiver: Box::new(expr),
    }
    .into()
}

// For checkbox, defaults can be given for each option, this method, takes option
// (`choice`), and the default value to put (`default`), and produces the following as
// an `Expr`:
// ```
// (choice.into(), default.into())`
// ```
fn make_checkbox_tuple(choice: syn::Expr, default: syn::Expr) -> syn::Expr {
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

fn parse_checkbox_choice(input: syn::parse::ParseStream) -> syn::Result<Choice> {
    let choice = input.parse()?;

    let choice = match choice {
        Choice::Choice(choice) if input.peek(Token![default]) => {
            input.parse::<Token![default]>()?;
            Choice::Choice(make_checkbox_tuple(choice, input.parse()?))
        }
        Choice::Choice(choice) => {
            let span = choice.span();
            Choice::Choice(make_checkbox_tuple(
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
