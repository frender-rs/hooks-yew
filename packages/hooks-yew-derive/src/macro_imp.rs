use std::borrow::Cow;

use darling::{FromMeta, ToTokens};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(Debug, Default, FromMeta)]
#[non_exhaustive]
#[darling(default)]
pub struct HookComponentArgs {
    /// Defaults to `::hooks_yew`
    pub hooks_yew_path: Option<syn::Path>,

    /// Defaults to `#hooks_yew_path::__private::yew`
    pub yew_path: Option<syn::Path>,

    /// Defaults to `#hooks_yew_path::__private::hooks_core`
    pub hooks_core_path: Option<syn::Path>,

    /// Defaults to `#hooks_yew_path::__private::hook_macro`, which is `::hooks::hook`.
    pub hook_macro_path: Option<syn::Path>,
}

impl HookComponentArgs {
    pub fn transform_item_fn(
        self,
        mut item_fn: syn::ItemFn,
    ) -> (TokenStream, Option<darling::Error>) {
        let mut errors = darling::error::Accumulator::default();

        let hooks_yew_path = self.hooks_yew_path.unwrap_or_else(|| syn::Path {
            leading_colon: Some(Default::default()),
            segments: syn::punctuated::Punctuated::from_iter([syn::PathSegment::from(
                syn::Ident::new("hooks_yew", Span::call_site()),
            )]),
        });

        let yew_path = self
            .yew_path
            .unwrap_or_else(|| private_path(&hooks_yew_path, "yew"));

        let hooks_core_path = self
            .hooks_core_path
            .unwrap_or_else(|| private_path(&hooks_yew_path, "hooks_core"));

        let hook_macro_path = self
            .hook_macro_path
            .unwrap_or_else(|| private_path(&hooks_yew_path, "hook_macro"));

        let span_fn_name = item_fn.sig.ident.span();

        let generics = std::mem::take(&mut item_fn.sig.generics);

        let vis = item_fn.vis.clone();

        let mut struct_attrs = Vec::with_capacity(item_fn.attrs.len());
        let mut fn_attrs = Vec::with_capacity(2);
        for attr in item_fn.attrs {
            if matches!(&attr.style, syn::AttrStyle::Outer) && !path_is_inline(&attr.path) {
                struct_attrs.push(attr)
            } else {
                // #[inline]
                // #![inner_attr]
                fn_attrs.push(attr)
            }
        }

        item_fn.attrs = fn_attrs;

        let (props_ty, args_is_empty) = {
            if item_fn.sig.inputs.is_empty() {
                item_fn
                    .sig
                    .inputs
                    .push_value(syn::FnArg::Typed(syn::PatType {
                        attrs: vec![],
                        pat: Box::new(syn::Pat::Wild(syn::PatWild {
                            attrs: vec![],
                            underscore_token: Default::default(),
                        })),
                        colon_token: Default::default(),
                        ty: Box::new(ref_type_tuple_0()),
                    }));

                (Cow::Owned(type_tuple_0()), true)
            } else {
                if item_fn.sig.inputs.len() > 1 {
                    errors.push(
                        darling::Error::custom("type_tuple_0() one optional argument is allowed")
                            .with_span(&item_fn.sig.inputs),
                    );
                }

                let first_arg = item_fn.sig.inputs.first().unwrap();
                let ty = match first_arg {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(pat_ty) => {
                        if let syn::Type::Reference(ty) = &*pat_ty.ty {
                            Some(&*ty.elem)
                        } else {
                            None
                        }
                    }
                };

                let ty = ty.map_or_else(
                    || {
                        errors.push(
                            darling::Error::custom("first argument must be `&Props`")
                                .with_span(first_arg),
                        );
                        Cow::Owned(type_tuple_0())
                    },
                    Cow::Borrowed,
                );
                (ty, false)
            }
        };

        let ident = std::mem::replace(
            &mut item_fn.sig.ident,
            syn::Ident::new(
                if args_is_empty {
                    "use_impl_html"
                } else {
                    "use_html"
                },
                span_fn_name,
            ),
        );

        let fn_ident = &item_fn.sig.ident;

        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

        let hook_attr = syn::Attribute {
            pound_token: Default::default(),
            style: syn::AttrStyle::Outer,
            bracket_token: Default::default(),
            path: hook_macro_path,
            tokens: {
                let hooks_core_path = syn::LitStr::new(
                    &hooks_core_path.to_token_stream().to_string(),
                    hooks_core_path.span(),
                );
                quote! { (hooks_core_path = #hooks_core_path) }
            },
        };

        let item_fn_use_html = if args_is_empty {
            Some(quote_spanned! { span_fn_name =>
                #hook_attr
                #[inline]
                #[allow(dead_code)]
                #vis fn use_html() -> #yew_path ::Html {
                    Self::use_impl_html(&())
                }
            })
        } else {
            None
        };

        item_fn.attrs.push(hook_attr);

        if args_is_empty {
            item_fn.vis = syn::Visibility::Inherited;
        }

        auto_fn_output_yew_html(&mut item_fn.sig.output, &yew_path);

        let ts = quote_spanned! { span_fn_name =>
            #(#struct_attrs)*
            #vis struct #ident #generics (
                #hooks_yew_path ::PinBoxDynHookComponent::<#props_ty>
            ) #where_clause;

            impl #impl_generics #ident #type_generics {
                #item_fn_use_html

                #item_fn
            }

            impl #yew_path ::Component for #impl_generics #ident #type_generics {
                type Message = ::core::primitive::bool;

                type Properties = #props_ty;

                #[inline]
                fn create(_: & #yew_path ::Context<Self>) -> Self {
                    Self(#hooks_yew_path ::PinBoxDynHookComponent::<#props_ty>::new(
                        ::std::boxed::Box::pin(Self:: #fn_ident ())
                    ))
                }

                #[inline]
                fn view(&self, ctx: & #yew_path ::Context<Self>) -> #yew_path ::Html {
                    self.0.view(ctx.props())
                }

                #[inline]
                fn update(&mut self, _: & #yew_path ::Context<Self>, msg: Self::Message) -> ::core::primitive::bool {
                    msg
                }

                #[inline]
                fn changed(&mut self, _: & #yew_path ::Context<Self>) -> bool {
                    self.0.changed()
                }

                #[inline]
                fn rendered(&mut self, ctx: & #yew_path ::Context<Self>, _: bool) {
                    self.0.rendered(ctx, |comp| &comp.0)
                }
            }
        };

        (ts, errors.finish().err())
    }
}

/// Auto fill elided return type
/// `fn MyComp() -> _` and `fn MyComp()` will be changed to
/// `fn MyComp() -> #yew_path ::Html`.
#[inline]
fn auto_fn_output_yew_html(output: &mut syn::ReturnType, yew_path: &syn::Path) {
    match output {
        out @ syn::ReturnType::Default => {
            *out = syn::ReturnType::Type(
                Default::default(),
                Box::new(type_yew_html(yew_path.clone(), out.span())),
            )
        }
        syn::ReturnType::Type(_, ty) => {
            if let syn::Type::Infer(ty_infer) = &**ty {
                **ty = type_yew_html(yew_path.clone(), ty_infer.span())
            }
        }
    }
}

fn path_is_inline(path: &syn::Path) -> bool {
    path.leading_colon.is_none() && path.segments.len() == 1 && !path.segments.trailing_punct() && {
        let seg = path.segments.first().unwrap();
        matches!(&seg.arguments, syn::PathArguments::None) && seg.ident == "inline"
    }
}

#[inline]
fn private_path(hooks_yew_path: &syn::Path, name: &str) -> syn::Path {
    let mut base = hooks_yew_path.clone();
    base.segments.push(syn::PathSegment::from(syn::Ident::new(
        "__private",
        Span::call_site(),
    )));
    base.segments.push(syn::PathSegment::from(syn::Ident::new(
        name,
        Span::call_site(),
    )));

    base
}

#[inline]
fn type_tuple_0() -> syn::Type {
    syn::Type::Tuple(syn::TypeTuple {
        paren_token: Default::default(),
        elems: Default::default(),
    })
}

#[inline]
fn ref_type_tuple_0() -> syn::Type {
    syn::Type::Reference(syn::TypeReference {
        and_token: Default::default(),
        lifetime: None,
        mutability: None,
        elem: Box::new(type_tuple_0()),
    })
}

#[inline]
fn type_yew_html(mut yew_path: syn::Path, span: Span) -> syn::Type {
    yew_path
        .segments
        .push(syn::PathSegment::from(syn::Ident::new("Html", span)));
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: yew_path,
    })
}
