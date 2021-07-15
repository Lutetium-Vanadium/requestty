use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse::Parse, spanned::Spanned};

use crate::helpers::*;

bitflags::bitflags! {
    pub struct BuilderMethods: u16 {
        const DEFAULT        = 0b00000_0001;
        const TRANSFORM      = 0b00000_0010;
        const VAL_FIL        = 0b00000_0100;
        const AUTO_COMPLETE  = 0b00000_1000;
        const LOOP_PAGE_SIZE = 0b00001_0000;
        const CHOICES        = 0b00010_0000;
        const MASK           = 0b00100_0000;
        const EXTENSION      = 0b01000_0000;
        const PLUGIN         = 0b10000_0000;
    }
}

#[derive(Clone, Copy)]
pub(crate) enum QuestionKind {
    Input,
    Int,
    Float,
    Confirm,
    Select,
    RawSelect,
    Expand,
    MultiSelect,
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
            QuestionKind::Select => "select",
            QuestionKind::RawSelect => "raw_select",
            QuestionKind::Expand => "expand",
            QuestionKind::MultiSelect => "multi_select",
            QuestionKind::Password => "password",
            QuestionKind::Editor => "editor",
            QuestionKind::Plugin => "plugin",
        }
    }

    fn get_builder_methods(&self) -> BuilderMethods {
        match *self {
            QuestionKind::Input => {
                BuilderMethods::DEFAULT
                    | BuilderMethods::TRANSFORM
                    | BuilderMethods::VAL_FIL
                    | BuilderMethods::AUTO_COMPLETE
                    | BuilderMethods::LOOP_PAGE_SIZE
            }
            QuestionKind::Int | QuestionKind::Float => {
                BuilderMethods::DEFAULT | BuilderMethods::TRANSFORM | BuilderMethods::VAL_FIL
            }
            QuestionKind::Confirm => BuilderMethods::DEFAULT | BuilderMethods::TRANSFORM,
            QuestionKind::Select | QuestionKind::RawSelect | QuestionKind::Expand => {
                BuilderMethods::DEFAULT
                    | BuilderMethods::TRANSFORM
                    | BuilderMethods::LOOP_PAGE_SIZE
                    | BuilderMethods::CHOICES
            }
            QuestionKind::MultiSelect => {
                BuilderMethods::TRANSFORM
                    | BuilderMethods::VAL_FIL
                    | BuilderMethods::LOOP_PAGE_SIZE
                    | BuilderMethods::CHOICES
            }
            QuestionKind::Password => {
                BuilderMethods::TRANSFORM | BuilderMethods::VAL_FIL | BuilderMethods::MASK
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

        let kind = if ident == "Input" {
            QuestionKind::Input
        } else if ident == "Int" {
            QuestionKind::Int
        } else if ident == "Float" {
            QuestionKind::Float
        } else if ident == "Confirm" {
            QuestionKind::Confirm
        } else if ident == "Select" {
            QuestionKind::Select
        } else if ident == "RawSelect" {
            QuestionKind::RawSelect
        } else if ident == "Expand" {
            QuestionKind::Expand
        } else if ident == "MultiSelect" {
            QuestionKind::MultiSelect
        } else if ident == "Password" {
            QuestionKind::Password
        } else if ident == "Editor" {
            QuestionKind::Editor
        } else if ident == "Plugin" {
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
    pub(crate) auto_complete: Option<syn::Expr>,

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
            auto_complete: None,

            choices: None,
            page_size: None,
            should_loop: None,

            mask: None,
            extension: None,

            plugin: None,
        }
    }
}

fn check_allowed(ident: &syn::Ident, kind: QuestionKind) -> syn::Result<()> {
    // default options which are always there
    if ident == "name" || ident == "message" || ident == "when" || ident == "ask_if_answered" {
        return Ok(());
    }

    let builder_method = if ident == "default" {
        BuilderMethods::DEFAULT
    } else if ident == "transform" {
        BuilderMethods::TRANSFORM
    } else if ident == "validate" || ident == "filter" {
        BuilderMethods::VAL_FIL
    } else if ident == "auto_complete" {
        BuilderMethods::AUTO_COMPLETE
    } else if ident == "choices" {
        BuilderMethods::CHOICES
    } else if ident == "page_size" || ident == "should_loop" {
        BuilderMethods::LOOP_PAGE_SIZE
    } else if ident == "mask" {
        BuilderMethods::MASK
    } else if ident == "extension" {
        BuilderMethods::EXTENSION
    } else if ident == "plugin" {
        BuilderMethods::PLUGIN
    } else {
        return Err(syn::Error::new(
            ident.span(),
            format!("unknown question option `{}`", ident),
        ));
    };

    if kind.get_builder_methods().contains(builder_method) {
        Ok(())
    } else {
        Err(syn::Error::new(
            ident.span(),
            format!("option `{}` does not exist for kind `{}`", ident, kind),
        ))
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

        while !content.is_empty() {
            let ident = content.parse::<syn::Ident>()?;

            check_allowed(&ident, kind)?;

            if ident == "name" {
                insert_non_dup(ident, &mut name, &content)?;
            } else if ident == "message" {
                insert_non_dup(ident, &mut opts.message, &content)?;
            } else if ident == "when" {
                insert_non_dup(ident, &mut opts.when, &content)?;
            } else if ident == "ask_if_answered" {
                insert_non_dup(ident, &mut opts.ask_if_answered, &content)?;
            } else if ident == "default" {
                insert_non_dup(ident, &mut opts.default, &content)?;
            } else if ident == "validate" {
                insert_non_dup(ident, &mut opts.validate, &content)?;
            } else if ident == "filter" {
                insert_non_dup(ident, &mut opts.filter, &content)?;
            } else if ident == "transform" {
                insert_non_dup(ident, &mut opts.transform, &content)?;
            } else if ident == "auto_complete" {
                insert_non_dup(ident, &mut opts.auto_complete, &content)?;
            } else if ident == "choices" {
                let parser = match kind {
                    QuestionKind::MultiSelect => Choices::parse_multi_select_choice,
                    _ => Choices::parse_choice,
                };

                insert_non_dup_parse(ident, &mut opts.choices, &content, parser)?;
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
                unreachable!("check_allowed should have taken care of this case.");
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
            name: name
                .ok_or_else(|| syn::Error::new(brace.span, "missing required option `name`"))?,
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
            let plugin = self
                .opts
                .plugin
                .as_ref()
                .expect("Parsing would error if no plugin was there");
            // If just the name was passed into Question::plugin, type errors associated
            // with its conversion to a string would take the span _including_ that of
            // plugin. Explicitly performing `String::from`, makes the error span due to
            // the `From` trait will show the span of the name only
            let name = quote_spanned! {
                name.span() => String::from(#name)
            };
            tokens.extend(quote_spanned! {
                plugin.span() => ::requestty::Question::plugin(#name, #plugin)
            });
            self.write_main_opts(tokens);
            tokens.extend(quote! { .build() });
            return;
        }

        let kind = syn::Ident::new(self.kind.as_str(), name.span());

        tokens.extend(quote_spanned! {
            name.span() => ::requestty::Question::#kind(#name)
        });

        self.write_main_opts(tokens);
        if let Some(ref default) = self.opts.default {
            tokens.extend(quote_spanned! { default.span() => .default(#default) });
        }
        if let Some(ref validate) = self.opts.validate {
            tokens.extend(quote_spanned! { validate.span() => .validate(#validate) });
        }
        if let Some(ref filter) = self.opts.filter {
            tokens.extend(quote_spanned! { filter.span() => .filter(#filter) });
        }
        if let Some(ref transform) = self.opts.transform {
            tokens.extend(quote_spanned! { transform.span() => .transform(#transform) });
        }
        if let Some(ref auto_complete) = self.opts.auto_complete {
            tokens
                .extend(quote_spanned! { auto_complete.span() => .auto_complete(#auto_complete) });
        }
        if let Some(ref choices) = self.opts.choices {
            tokens.extend(match self.kind {
                QuestionKind::MultiSelect => {
                    quote_spanned! { choices.span() => .choices_with_default(#choices) }
                }
                _ => quote_spanned! { choices.span() => .choices(#choices) },
            });
        }
        if let Some(ref page_size) = self.opts.page_size {
            tokens.extend(quote_spanned! { page_size.span() => .page_size(#page_size) });
        }
        if let Some(ref should_loop) = self.opts.should_loop {
            tokens.extend(quote_spanned! { should_loop.span() => .should_loop(#should_loop) });
        }
        if let Some(ref mask) = self.opts.mask {
            tokens.extend(quote_spanned! { mask.span() => .mask(#mask) });
        }
        if let Some(ref extension) = self.opts.extension {
            tokens.extend(quote_spanned! { extension.span() => .extension(#extension) });
        }
        tokens.extend(quote! { .build() });
    }
}