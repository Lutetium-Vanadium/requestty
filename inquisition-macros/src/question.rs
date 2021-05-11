use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse::Parse, spanned::Spanned, Token};

use crate::helpers::*;

bitflags::bitflags! {
    pub struct BuilderMethods: u8 {
        const DEFAULT   = 0b000_0001;
        const TRANSFORM = 0b000_0010;
        const VAL_FIL   = 0b000_0100;
        const LIST      = 0b000_1000;
        const MASK      = 0b001_0000;
        const EXTENSION   = 0b010_0000;
        const PLUGIN    = 0b100_0000;
    }
}

#[derive(Clone, Copy)]
pub(crate) enum QuestionKind {
    Input,
    Int,
    Float,
    Confirm,
    List,
    Rawlist,
    Expand,
    Checkbox,
    Password,
    Editor,
    Plugin,
}

impl QuestionKind {
    fn as_str(&self) -> &str {
        match self {
            QuestionKind::Input => "input",
            QuestionKind::Int => "int",
            QuestionKind::Float => "float",
            QuestionKind::Confirm => "confirm",
            QuestionKind::List => "list",
            QuestionKind::Rawlist => "rawlist",
            QuestionKind::Expand => "expand",
            QuestionKind::Checkbox => "checkbox",
            QuestionKind::Password => "password",
            QuestionKind::Editor => "editor",
            QuestionKind::Plugin => "plugin",
        }
    }

    fn get_builder_methods(&self) -> BuilderMethods {
        match *self {
            QuestionKind::Input | QuestionKind::Int | QuestionKind::Float => {
                BuilderMethods::DEFAULT
                    | BuilderMethods::TRANSFORM
                    | BuilderMethods::VAL_FIL
            }
            QuestionKind::Confirm => {
                BuilderMethods::DEFAULT | BuilderMethods::TRANSFORM
            }
            QuestionKind::List | QuestionKind::Rawlist | QuestionKind::Expand => {
                BuilderMethods::DEFAULT
                    | BuilderMethods::TRANSFORM
                    | BuilderMethods::LIST
            }
            QuestionKind::Checkbox => {
                BuilderMethods::DEFAULT
                    | BuilderMethods::TRANSFORM
                    | BuilderMethods::VAL_FIL
                    | BuilderMethods::LIST
            }
            QuestionKind::Password => {
                BuilderMethods::TRANSFORM
                    | BuilderMethods::VAL_FIL
                    | BuilderMethods::MASK
            }
            QuestionKind::Editor => {
                BuilderMethods::DEFAULT
                    | BuilderMethods::TRANSFORM
                    | BuilderMethods::VAL_FIL
                    | BuilderMethods::EXTENSION
            }
            QuestionKind::Plugin => BuilderMethods::PLUGIN,
        }
    }
}

impl Parse for QuestionKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;

        let kind = if ident == "input" {
            QuestionKind::Input
        } else if ident == "int" {
            QuestionKind::Int
        } else if ident == "float" {
            QuestionKind::Float
        } else if ident == "confirm" {
            QuestionKind::Confirm
        } else if ident == "list" {
            QuestionKind::List
        } else if ident == "rawlist" {
            QuestionKind::Rawlist
        } else if ident == "expand" {
            QuestionKind::Expand
        } else if ident == "checkbox" {
            QuestionKind::Checkbox
        } else if ident == "password" {
            QuestionKind::Password
        } else if ident == "editor" {
            QuestionKind::Editor
        } else if ident == "plugin" {
            QuestionKind::Plugin
        } else {
            return Err(syn::Error::new(
                ident.span(),
                format!("unknown question kind {}", ident),
            ));
        };

        Ok(kind)
    }
}

impl fmt::Display for QuestionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub(crate) struct QuestionOpts {
    pub(crate) message: Option<syn::Expr>,
    pub(crate) when: Option<syn::Expr>,
    pub(crate) ask_if_answered: Option<syn::Expr>,

    pub(crate) default: Option<syn::Expr>,

    pub(crate) validate: Option<syn::Expr>,
    pub(crate) filter: Option<syn::Expr>,
    pub(crate) transform: Option<syn::Expr>,
    pub(crate) validate_async: Option<syn::Expr>,
    pub(crate) filter_async: Option<syn::Expr>,
    pub(crate) transform_async: Option<syn::Expr>,

    pub(crate) choices: Option<Choices>,
    pub(crate) page_size: Option<syn::Expr>,
    pub(crate) should_loop: Option<syn::Expr>,

    pub(crate) mask: Option<syn::Expr>,
    pub(crate) extension: Option<syn::Expr>,

    pub(crate) plugin: Option<syn::Expr>,
}

impl Default for QuestionOpts {
    fn default() -> Self {
        Self {
            message: None,
            when: None,
            ask_if_answered: None,

            default: None,

            validate: None,
            filter: None,
            transform: None,
            validate_async: None,
            filter_async: None,
            transform_async: None,

            choices: None,
            page_size: None,
            should_loop: None,

            mask: None,
            extension: None,

            plugin: None,
        }
    }
}

/// Checks if a _valid_ ident is disallowed
fn check_disallowed(
    ident: &syn::Ident,
    kind: QuestionKind,
    allowed: BuilderMethods,
) -> syn::Result<()> {
    #[rustfmt::skip]
    fn disallowed(ident: &syn::Ident, allowed: BuilderMethods) -> bool {
         (ident == "default" &&
          !allowed.contains(BuilderMethods::DEFAULT)) ||

        ((ident == "transform_async" ||
          ident == "transform") &&
          !allowed.contains(BuilderMethods::TRANSFORM)) ||

        ((ident == "validate_async" ||
          ident == "validate" ||
          ident == "filter" ||
          ident == "filter_async") &&
          !allowed.contains(BuilderMethods::VAL_FIL)) ||

        ((ident == "choices" ||
          ident == "page_size" ||
          ident == "should_loop") &&
          !allowed.contains(BuilderMethods::LIST)) ||

         (ident == "mask" &&
          !allowed.contains(BuilderMethods::MASK)) ||

         (ident == "extension" &&
          !allowed.contains(BuilderMethods::EXTENSION)) ||

         (ident == "plugin" &&
          !allowed.contains(BuilderMethods::PLUGIN))
    }

    if disallowed(ident, allowed) {
        Err(syn::Error::new(
            ident.span(),
            format!("option `{}` does not exist for kind `{}`", ident, kind),
        ))
    } else {
        Ok(())
    }
}

pub(crate) struct Question {
    pub(crate) kind: QuestionKind,
    pub(crate) name: syn::Expr,
    pub(crate) opts: QuestionOpts,
}

impl Parse for Question {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kind = QuestionKind::parse(input)?;
        let content;
        let brace = syn::braced!(content in input);

        let mut opts = QuestionOpts::default();
        let mut name = None;

        let allowed_methods = kind.get_builder_methods();

        while !content.is_empty() {
            if content.peek(Token![async]) {
                let asynct = content.parse::<Token![async]>()?;
                let ident = content.parse::<syn::Ident>()?;

                content.parse::<Token![:]>()?;

                let full_ident_str = format!("{}_async", ident);
                let full_ident = syn::Ident::new(
                    &full_ident_str,
                    asynct
                        .span
                        .join(ident.span())
                        .unwrap_or_else(|| ident.span()),
                );

                check_disallowed(&full_ident, kind, allowed_methods)?;
                if ident == "validate" {
                    insert_non_dup(full_ident, &mut opts.validate_async, &content)?;
                } else if ident == "filter" {
                    insert_non_dup(full_ident, &mut opts.filter_async, &content)?;
                } else if ident == "transform" {
                    insert_non_dup(full_ident, &mut opts.transform_async, &content)?;
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown question option `{}`", full_ident_str),
                    ));
                }
            } else {
                let ident = content.parse::<syn::Ident>()?;

                content.parse::<Token![:]>()?;

                // it is not an issue if ident doesn't correspond to valid option
                // since check_allowed only checks if valid idents are disallowed
                check_disallowed(&ident, kind, allowed_methods)?;

                // default options which are always there
                if ident == "name" {
                    insert_non_dup(ident, &mut name, &content)?;
                } else if ident == "message" {
                    insert_non_dup(ident, &mut opts.message, &content)?;
                } else if ident == "when" {
                    insert_non_dup(ident, &mut opts.when, &content)?;
                } else if ident == "ask_if_answered" {
                    insert_non_dup(ident, &mut opts.ask_if_answered, &content)?;
                } else {
                    // the rest may or may not be there, so must be checked
                    // it is not an issue if ident doesn't correspond to valid option
                    // since check_allowed only checks if valid idents are disallowed
                    check_disallowed(&ident, kind, allowed_methods)?;

                    if ident == "default" {
                        insert_non_dup(ident, &mut opts.default, &content)?;
                    } else if ident == "validate" {
                        insert_non_dup(ident, &mut opts.validate, &content)?;
                    } else if ident == "filter" {
                        insert_non_dup(ident, &mut opts.filter, &content)?;
                    } else if ident == "transform" {
                        insert_non_dup(ident, &mut opts.transform, &content)?;
                    } else if ident == "choices" {
                        match kind {
                            QuestionKind::Checkbox => insert_non_dup_parse(
                                ident,
                                &mut opts.choices,
                                &content,
                                Choices::parse_checkbox_choice,
                            )?,
                            _ => insert_non_dup_parse(
                                ident,
                                &mut opts.choices,
                                &content,
                                Choices::parse_choice,
                            )?,
                        }
                    } else if ident == "page_size" {
                        insert_non_dup(ident, &mut opts.page_size, &content)?;
                    } else if ident == "should_loop" {
                        insert_non_dup(ident, &mut opts.should_loop, &content)?;
                    } else if ident == "mask" {
                        insert_non_dup(ident, &mut opts.mask, &content)?;
                    } else if ident == "extension" {
                        insert_non_dup(ident, &mut opts.extension, &content)?;
                    } else if ident == "plugin" {
                        insert_non_dup(ident, &mut opts.plugin, &content)?;
                    } else {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("unknown question option `{}`", ident),
                        ));
                    }
                }
            }

            if parse_optional_comma(&content)?.is_none() {
                break;
            }
        }

        if let QuestionKind::Plugin = kind {
            if opts.plugin.is_none() {
                return Err(syn::Error::new(
                    brace.span,
                    "missing required option `plugin`",
                ));
            }
        }

        Ok(Self {
            kind,
            name: name.ok_or_else(|| {
                syn::Error::new(brace.span, "missing required option `name`")
            })?,
            opts,
        })
    }
}

impl Question {
    fn write_main_opts(&self, tokens: &mut TokenStream) {
        if let Some(ref message) = self.opts.message {
            tokens.extend(quote_spanned! { message.span() => .message(#message) });
        }
        if let Some(ref when) = self.opts.when {
            tokens.extend(quote_spanned! { when.span() => .when(#when) });
        }
        if let Some(ref ask_if_answered) = self.opts.ask_if_answered {
            tokens.extend(quote_spanned! {
                ask_if_answered.span() => .ask_if_answered(#ask_if_answered)
            });
        }
    }
}

impl quote::ToTokens for Question {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;

        if let QuestionKind::Plugin = self.kind {
            let plugin = self.opts.plugin.as_ref().unwrap();
            // If just the name was passed into Question::plugin, type errors associated
            // with its conversion to a string would take the span _including_ that of
            // plugin. Explicitly performing `String::from`, makes the error span due to
            // the `From` trait will show the span of the name only
            let name = quote_spanned! {
                name.span() => String::from(#name)
            };
            tokens.extend(quote_spanned! {
                plugin.span() => ::inquisition::Question::plugin(#name, #plugin)
            });
            self.write_main_opts(tokens);
            tokens.extend(quote! { .build() });
            return;
        }

        let kind = syn::Ident::new(self.kind.as_str(), name.span());

        tokens.extend(quote_spanned! {
            name.span() => ::inquisition::Question::#kind(#name)
        });

        self.write_main_opts(tokens);
        if let Some(ref default) = self.opts.default {
            tokens.extend(quote_spanned! { default.span() => .default(#default) });
        }
        if let Some(ref validate) = self.opts.validate {
            tokens
                .extend(quote_spanned! { validate.span() => .validate(#validate) });
        }
        if let Some(ref validate_async) = self.opts.validate_async {
            tokens.extend(quote_spanned! {
                validate_async.span() => .validate_async(#validate_async)
            });
        }
        if let Some(ref filter) = self.opts.filter {
            tokens.extend(quote_spanned! { filter.span() => .filter(#filter) });
        }
        if let Some(ref filter_async) = self.opts.filter_async {
            tokens.extend(quote_spanned! {
                filter_async.span() => .filter_async(#filter_async)
            });
        }
        if let Some(ref transform) = self.opts.transform {
            tokens.extend(
                quote_spanned! { transform.span() => .transform(#transform) },
            );
        }
        if let Some(ref transform_async) = self.opts.transform_async {
            tokens.extend(quote_spanned! {
                transform_async.span() => .transform_async(#transform_async)
            });
        }
        if let Some(ref choices) = self.opts.choices {
            tokens.extend(match self.kind {
                QuestionKind::Checkbox => {
                    quote_spanned! { choices.span() => .choices_with_default(#choices) }
                }
                _ => quote_spanned! { choices.span() => .choices(#choices) },
            });
        }
        if let Some(ref page_size) = self.opts.page_size {
            tokens.extend(
                quote_spanned! { page_size.span() => .page_size(#page_size) },
            );
        }
        if let Some(ref should_loop) = self.opts.should_loop {
            tokens.extend(
                quote_spanned! { should_loop.span() => .should_loop(#should_loop) },
            );
        }
        if let Some(ref mask) = self.opts.mask {
            tokens.extend(quote_spanned! { mask.span() => .mask(#mask) });
        }
        if let Some(ref extension) = self.opts.extension {
            tokens.extend(
                quote_spanned! { extension.span() => .extension(#extension) },
            );
        }
        tokens.extend(quote! { .build() });
    }
}
