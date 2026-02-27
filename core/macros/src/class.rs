use crate::utils::{
    RenameScheme, SpannedResult, error, take_error_from_attrs, take_length_from_attrs,
    take_name_value_attr, take_path_attr,
};
use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{
    Attribute, ConstParam, Expr, FnArg, GenericParam, Ident, ImplItemFn, ItemImpl, LifetimeParam,
    Lit, Meta, MetaNameValue, PatType, Receiver, ReturnType, Signature, Token, Type, TypeParam,
};

/// A function representation. Takes a function from the AST and remember its name, length and
/// how its body should be in the output AST.
/// There are three types of functions: Constructors, Methods and Accessors (setter/getter).
///
/// This is not an enum for simplicity. The body is dependent on how this was created.
pub(crate) struct Function {
    /// The name of the function. Can be overridden with `#[boa(rename = "...")]`.
    name: String,

    /// The length of the function in JavaScript. Can be overridden with `#[boa(length = ...)]`.
    length: usize,

    /// The body of the function serialized. This depends highly on the type of function.
    body: TokenStream2,

    /// Whether a receiver was found.
    is_static: bool,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("length", &self.length)
            .field("is_static", &self.is_static)
            .field("body", &self.body.to_string())
            .finish()
    }
}

impl Function {
    /// Serializes the `self` argument declaration and call.
    fn arg_self_from_receiver(
        receiver: &mut Receiver,
        class_ty: &Type,
    ) -> SpannedResult<(TokenStream2, TokenStream2)> {
        let err = take_error_from_attrs(&mut receiver.attrs)?
            .unwrap_or("Invalid class for type".to_string());

        // `&mut self`
        let downcast = if receiver.mutability.is_some() {
            quote! {
                let object = this.as_object();
                let self_ = &mut *object.as_ref().and_then(|o| o.downcast_mut::< #class_ty >())
                    .ok_or( boa_engine::js_error!( #err ))?;
            }
        } else {
            quote! {
                let object = this.as_object();
                let self_ = &*object.as_ref().and_then(|o| o.downcast_ref::< #class_ty >())
                    .ok_or( boa_engine::js_error!( #err ))?;
            }
        };

        Ok((downcast, quote! { self_ }))
    }

    /// Serializes an argument of form `pat: Type` into its declaration and call. Also returns
    /// whether we should increment the length.
    #[allow(clippy::unnecessary_wraps)]
    fn arg_from_pat_type(
        pat_type: &mut PatType,
        i: usize,
    ) -> SpannedResult<(bool, TokenStream2, TokenStream2)> {
        let ty = pat_type.ty.as_ref();
        let ident = Ident::new(&format!("boa_arg_{i}"), Span2::call_site());

        // Find out if it's a boa context.
        let is_context = match ty {
            Type::Reference(syn::TypeReference {
                elem,
                ..
            }) => match elem.as_ref() {
                Type::Path(syn::TypePath { qself: _, path }) => {
                    if let Some(maybe_ctx) = path.segments.last() {
                        maybe_ctx.ident == "Context"
                    } else {
                        false
                    }
                }
                _ => take_path_attr(&mut pat_type.attrs, "context"),
            },
            _ => false,
        };

        if is_context {
            Ok((true, quote! {}, quote! { context }))
        } else {
            Ok((
                false,
                quote! {
                    let (#ident, rest): (#ty, &[boa_engine::JsValue]) =
                        boa_engine::interop::TryFromJsArgument::try_from_js_argument( this, rest, context )?;
                },
                quote! { #ident },
            ))
        }
    }

    pub(crate) fn from_sig(
        name: String,
        has_explicit_method: bool,
        has_explicit_static: bool,
        attrs: &mut Vec<Attribute>,
        sig: &mut Signature,
        class_ty: Option<&Type>,
    ) -> SpannedResult<Self> {
        // The amount of arguments that aren't really arguments in JavaScript,
        // e.g. `self`, `&Context`, etc.
        let mut not_param_count = 0;
        let (args_decl, args_call): (Vec<TokenStream2>, Vec<TokenStream2>) = sig
            .inputs
            .iter_mut()
            .enumerate()
            .map(|(i, a)| match a {
                FnArg::Receiver(receiver) => {
                    not_param_count += 1;
                    if let Some(cty) = class_ty {
                        Self::arg_self_from_receiver(receiver, cty)
                    } else {
                        error(receiver, "Invalid context for using a receiver.")
                    }
                }
                FnArg::Typed(ty) => {
                    let (incr, decl, call) = Self::arg_from_pat_type(ty, i)?;
                    if incr {
                        not_param_count += 1;
                    }
                    Ok((decl, call))
                }
            })
            .collect::<SpannedResult<_>>()?;

        let length = take_length_from_attrs(attrs)?.unwrap_or(args_decl.len() - not_param_count);

        let fn_name = &sig.ident;

        let generics = if sig.generics.params.is_empty() {
            quote! {}
        } else {
            let generics = sig
                .generics
                .params
                .iter()
                .map(|param| match param {
                    GenericParam::Type(TypeParam { ident, .. })
                    | GenericParam::Const(ConstParam { ident, .. }) => {
                        quote! { #ident }
                    }
                    GenericParam::Lifetime(LifetimeParam { lifetime, .. }) => {
                        quote! { #lifetime }
                    }
                })
                .collect::<TokenStream2>();
            quote! { :: < #generics > }
        };

        // A method is static if it has the `#[boa(static)]` attribute, or if it is
        // not a method and doesn't contain `self`.
        let is_static = has_explicit_static || !(has_explicit_method || not_param_count > 0);

        // If this is a scoped function to a type (e.g. inside an `impl` block),
        let scope = if class_ty.is_some() {
            quote! { Self :: }
        } else {
            quote! {}
        };

        Ok(Self {
            length,
            name,
            body: quote! {
                |   this: &boa_engine::JsValue,
                    args: &[boa_engine::JsValue],
                    context: &boa_engine::Context
                | -> boa_engine::JsResult<boa_engine::JsValue> {
                    let rest = args;
                    #(#args_decl)*

                    let result = #scope #fn_name #generics ( #(#args_call),* );
                    boa_engine::TryIntoJsResult::try_into_js_result(result, context)
                }
            },
            is_static,
        })
    }

    /// Create a `Function` from its name,
    fn method(
        name: String,
        has_explicit_method: bool,
        has_explicit_static: bool,
        fn_: &mut ImplItemFn,
        class_ty: Option<&Type>,
    ) -> SpannedResult<Self> {
        if fn_.sig.asyncness.is_some() {
            error(&fn_.sig.asyncness, "Async methods are not supported.")?;
        }

        if !fn_.sig.generics.params.is_empty() {
            error(&fn_.sig.generics, "Generic methods are not supported.")?;
        }

        Self::from_sig(
            name,
            has_explicit_method,
            has_explicit_static,
            &mut fn_.attrs,
            &mut fn_.sig,
            class_ty,
        )
    }

    fn getter(name: String, fn_: &mut ImplItemFn, class_ty: &Type) -> SpannedResult<Self> {
        Self::method(name, false, true, fn_, Some(class_ty))
    }

    fn setter(name: String, fn_: &mut ImplItemFn, class_ty: &Type) -> SpannedResult<Self> {
        Self::method(name, false, true, fn_, Some(class_ty))
    }

    fn constructor(fn_: &mut ImplItemFn, _class_ty: &Type) -> SpannedResult<Self> {
        if fn_.sig.asyncness.is_some() {
            error(&fn_.sig.asyncness, "Async methods are not supported.")?;
        }

        if !fn_.sig.generics.params.is_empty() {
            error(&fn_.sig.generics, "Generic methods are not supported.")?;
        }

        let (args_decl, args_call): (Vec<TokenStream2>, Vec<TokenStream2>) = fn_
            .sig
            .inputs
            .iter_mut()
            .enumerate()
            .map(|(i, a)| match a {
                FnArg::Receiver(receiver) => error(receiver, "Constructors cannot use 'self'"),
                FnArg::Typed(ty) => {
                    let (_, decl, call) = Self::arg_from_pat_type(ty, i)?;
                    Ok((decl, call))
                }
            })
            .collect::<SpannedResult<_>>()?;

        let length = take_length_from_attrs(&mut fn_.attrs)?.unwrap_or(args_decl.len());
        let fn_name = &fn_.sig.ident;

        // Does the function return Result<_> or JsResult<_>? If so, Into JsResult (to
        // transform the error. If not, return Ok(_).
        let return_statement = match &fn_.sig.output {
            ReturnType::Default => quote! { Default::default() },
            ReturnType::Type(_, ty) => {
                if let Type::Path(path) = ty.as_ref() {
                    let Some(t) = path.path.segments.last() else {
                        return error(&fn_.sig.output, "Cannot infer return type.");
                    };
                    if t.ident == "Self" {
                        quote! { Ok(result) }
                    } else if t.ident == "JsResult" {
                        quote! { result.into() }
                    } else {
                        return error(
                            &fn_.sig.output,
                            "Invalid return type: constructors should return Self or JsResult<Self>.",
                        );
                    }
                } else {
                    quote! { Ok(result) }
                }
            }
        };

        Ok(Self {
            length,
            name: String::new(),
            body: quote! {
                let rest = args;
                #(#args_decl)*

                let result = Self:: #fn_name ( #(#args_call),* );
                #return_statement
            },
            is_static: false,
        })
    }

    pub(crate) fn body(&self) -> &TokenStream2 {
        &self.body
    }
}

#[derive(Debug, Default)]
struct Accessor {
    getter: Option<Function>,
    setter: Option<Function>,
}

impl Accessor {
    fn set_getter(
        &mut self,
        name: String,
        fn_: &mut ImplItemFn,
        class_ty: &Type,
    ) -> SpannedResult<()> {
        if self.getter.is_some() {
            error(fn_, "Getter for property {name:?} already declared.")
        } else {
            let getter = Function::getter(name, fn_, class_ty)?;
            self.getter = Some(getter);
            Ok(())
        }
    }

    fn set_setter(
        &mut self,
        name: String,
        fn_: &mut ImplItemFn,
        class_ty: &Type,
    ) -> SpannedResult<()> {
        if self.setter.is_some() {
            error(
                fn_,
                format!("Setter for property {name:?} already declared."),
            )
        } else {
            let setter = Function::setter(name, fn_, class_ty)?;
            self.setter = Some(setter);
            Ok(())
        }
    }

    fn body(&self) -> TokenStream2 {
        let Some(name) = self
            .getter
            .as_ref()
            .map_or_else(|| self.setter.as_ref().map(|s| &s.name), |g| Some(&g.name))
        else {
            return quote! {};
        };
        let getter = if let Some(getter) = self.getter.as_ref() {
            let body = getter.body.clone();
            quote! {
                Some(
                    boa_engine::NativeFunction::from_fn_ptr( #body )
                        .to_js_function(builder.context().realm())
                )
            }
        } else {
            quote! { None }
        };
        let setter = if let Some(setter) = self.setter.as_ref() {
            let body = setter.body.clone();
            quote! {
                Some(
                    boa_engine::NativeFunction::from_fn_ptr( #body )
                        .to_js_function(builder.context().realm())
                )
            }
        } else {
            quote! { None }
        };

        quote! {
            {
                let g = #getter;
                let s = #setter;
                builder.accessor(
                    boa_engine::js_string!( #name ),
                    g,
                    s,
                    boa_engine::property::Attribute::CONFIGURABLE
                        | boa_engine::property::Attribute::NON_ENUMERABLE,
                );
            }
        }
    }
}

/// A visitor for the `impl X { ... }` block. This will record all top-level functions
/// and create endpoints for the JavaScript class.
#[derive(Debug)]
struct ClassVisitor {
    renaming: RenameScheme,

    // The type name for this class.
    type_: Type,

    // Whether we detected a constructor while visiting.
    constructor: Option<Function>,

    // All static functions recorded.
    statics: Vec<Function>,

    // All methods recorded.
    methods: Vec<Function>,

    // All accessors (getters and/or setters) with their names.
    accessors: BTreeMap<String, Accessor>,

    // All errors we found along the way.
    errors: Option<syn::Error>,
}

impl ClassVisitor {
    fn new(renaming: RenameScheme, type_: Type) -> Self {
        Self {
            renaming,
            type_,
            constructor: None,
            statics: Vec::new(),
            methods: Vec::new(),
            accessors: BTreeMap::default(),
            errors: None,
        }
    }

    fn name_of(&self, fn_: &mut ImplItemFn) -> SpannedResult<String> {
        take_name_value_attr(&mut fn_.attrs, "rename").map_or_else(
            || Ok(self.renaming.rename(fn_.sig.ident.to_string())),
            |nv| match &nv {
                Lit::Str(s) => Ok(s.value()),
                _ => error(&nv, "Invalid attribute value literal"),
            },
        )
    }

    fn method(
        &mut self,
        explicit_method: bool,
        explicit_static: bool,
        fn_: &mut ImplItemFn,
    ) -> SpannedResult<()> {
        let name = self.name_of(fn_)?;
        let f = Function::method(
            name,
            explicit_method,
            explicit_static,
            fn_,
            Some(&self.type_),
        )?;

        if f.is_static {
            self.statics.push(f);
        } else {
            self.methods.push(f);
        }

        Ok(())
    }

    fn getter(&mut self, fn_: &mut ImplItemFn) -> SpannedResult<()> {
        let name = self.name_of(fn_)?;
        self.accessors
            .entry(name.clone())
            .or_default()
            .set_getter(name, fn_, &self.type_)?;

        Ok(())
    }

    fn setter(&mut self, fn_: &mut ImplItemFn) -> SpannedResult<()> {
        let name = self.name_of(fn_)?;
        self.accessors
            .entry(name.clone())
            .or_default()
            .set_setter(name, fn_, &self.type_)?;
        Ok(())
    }

    fn constructor(&mut self, fn_: &mut ImplItemFn) -> SpannedResult<()> {
        self.constructor = Some(Function::constructor(fn_, &self.type_)?);
        Ok(())
    }

    /// Add an error to list of errors we are recording along the way. Errors are handled
    /// at the end of the process, so this combines all errors.
    #[allow(clippy::needless_pass_by_value)]
    fn error(&mut self, node: impl Spanned, message: impl Display) {
        let error = syn::Error::new(node.span(), message);

        match &mut self.errors {
            None => {
                self.errors = Some(error);
            }
            Some(e) => {
                e.combine(error);
            }
        }
    }

    /// Serialize the `boa_engine::Class` implementation into a token stream.
    fn serialize_class_impl(&self, class_ty: &Type, class_name: &str) -> TokenStream2 {
        let arg_count = self.constructor.as_ref().map_or(0, |c| c.length);

        let accessors = self.accessors.values().map(Accessor::body);

        let builder_methods = self.methods.iter().map(|m| {
            let name_str = m.name.as_str();
            let length = m.length;
            let body = &m.body;

            quote! {
                builder.method(
                    boa_engine::js_string!( #name_str ),
                    #length,
                    boa_engine::NativeFunction::from_fn_ptr(
                        #body
                    )
                );
            }
        });

        let builder_statics = self.statics.iter().map(|m| {
            let name_str = m.name.as_str();
            let length = m.length;
            let body = &m.body;

            quote! {
                builder.static_method(
                    boa_engine::js_string!( #name_str ),
                    #length,
                    boa_engine::NativeFunction::from_fn_ptr(
                        #body
                    )
                );
            }
        });

        let constructor_body = self.constructor.as_ref().map_or_else(
            || {
                quote! {
                    Ok(Default::default())
                }
            },
            |c| c.body.clone(),
        );

        quote! {
            impl boa_engine::class::Class for #class_ty {
                const NAME: &'static str = #class_name;
                const LENGTH: usize = #arg_count;

                fn data_constructor(
                    this: &boa_engine::JsValue,
                    args: &[boa_engine::JsValue],
                    context: &boa_engine::Context
                ) -> boa_engine::JsResult<Self> {
                    #constructor_body
                }

                fn init(builder: &mut boa_engine::class::ClassBuilder) -> boa_engine::JsResult<()> {
                    // Add all statics.
                    #(#builder_statics)*

                    // Add all accessors.
                    #(#accessors)*

                    // Add all methods to the class.
                    #(#builder_methods)*

                    Ok(())
                }
            }
        }
    }
}

impl VisitMut for ClassVisitor {
    // Allow similar names as there are no better ways to name `getter` and `setter`.
    #[allow(clippy::similar_names)]
    fn visit_impl_item_fn_mut(&mut self, item: &mut ImplItemFn) {
        // If there's a `boa` argument, parse it.
        let has_ctor_attr = take_path_attr(&mut item.attrs, "constructor");
        let has_getter_attr = take_path_attr(&mut item.attrs, "getter");
        let has_setter_attr = take_path_attr(&mut item.attrs, "setter");
        let has_method_attr = take_path_attr(&mut item.attrs, "method");
        let has_static_attr = take_path_attr(&mut item.attrs, "static");

        if has_getter_attr && let Err((span, msg)) = self.getter(item) {
            self.error(span, msg);
        }

        if has_setter_attr && let Err((span, msg)) = self.setter(item) {
            self.error(span, msg);
        }

        if has_ctor_attr && let Err((span, msg)) = self.constructor(item) {
            self.error(span, msg);
        }

        // A function is a method if it has a `#[boa(method)]` attribute or has no
        // method-type related attributes.
        if (has_static_attr
            || has_method_attr
            || !(has_getter_attr || has_ctor_attr || has_setter_attr))
            && let Err((span, msg)) = self.method(has_method_attr, has_static_attr, item)
        {
            self.error(span, msg);
        }

        syn::visit_mut::visit_impl_item_fn_mut(self, item);
    }
}

#[derive(Debug)]
struct ClassArguments {
    name: Option<String>,
}

impl Parse for ClassArguments {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let args: Punctuated<Meta, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut name = None;

        for arg in &args {
            match arg {
                Meta::NameValue(MetaNameValue {
                    path,
                    value: Expr::Lit(lit),
                    ..
                }) if path.is_ident("rename") => {
                    name = Some(match &lit.lit {
                        Lit::Str(s) => Ok(s.value()),
                        _ => Err(syn::Error::new(lit.span(), "Expected a string literal")),
                    }?);
                }
                _ => return Err(syn::Error::new(arg.span(), "Unrecognize argument.")),
            }
        }

        Ok(Self { name })
    }
}

pub(crate) fn class_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute arguments.
    let args = syn::parse_macro_input!(attr as ClassArguments);

    // Parse the input.
    let mut impl_ = syn::parse_macro_input!(input as ItemImpl);

    let renaming = match RenameScheme::from_named_attrs(&mut impl_.attrs, "rename_all") {
        Ok(Some(r)) => r,
        Ok(None) => RenameScheme::CamelCase,
        Err((span, msg)) => {
            return syn::Error::new(span, msg).to_compile_error().into();
        }
    };

    // Get all methods from the input.
    let mut visitor = ClassVisitor::new(renaming, impl_.self_ty.as_ref().clone());
    syn::visit_mut::visit_item_impl_mut(&mut visitor, &mut impl_);

    if let Some(err) = visitor.errors {
        return err.to_compile_error().into();
    }

    let Type::Path(pa) = impl_.self_ty.as_ref() else {
        return syn::Error::new(impl_.span(), "Impossible to find the name of the class.")
            .to_compile_error()
            .into();
    };
    let Some(name) = args
        .name
        .or_else(|| pa.path.get_ident().map(ToString::to_string))
    else {
        return syn::Error::new(pa.span(), "Impossible to find the name of the class.")
            .to_compile_error()
            .into();
    };

    let class_impl = visitor.serialize_class_impl(&impl_.self_ty, &name);

    let debug = take_path_attr(&mut impl_.attrs, "debug");

    let tokens = quote! {
        // Keep the original implementation as is.
        // Add `#[allow(clippy::needless_pass_by_value)]`, as clippy can complaint when
        // using passing-by-value but not using it in the body, except that we cannot
        // convert the types if we pass by reference.
        #[allow(clippy::needless_pass_by_value)]
        #impl_

        // The boa_engine::Class implementation.
        #class_impl
    };

    #[allow(clippy::print_stderr)]
    if debug {
        eprintln!("---------\n{tokens}\n---------\n");
    }

    tokens.into()
}
