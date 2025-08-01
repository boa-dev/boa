use crate::class::Function;
use crate::utils::{RenameScheme, SpannedResult, error, take_name_value_string, take_path_attr};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro,
    ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse,
};

#[derive(Debug)]
struct ModuleArguments {}

impl Parse for ModuleArguments {
    fn parse(_input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {})
    }
}

fn const_item(
    c: &mut ItemConst,
    renaming: RenameScheme,
) -> SpannedResult<(String, TokenStream2, TokenStream2)> {
    let ident = &c.ident;
    let name = take_name_value_string(&mut c.attrs, "rename")?
        .unwrap_or_else(|| renaming.rename(ident.to_string()));

    Ok((
        name.clone(),
        quote! {
            m.set_export( &boa_engine::js_string!( #name ), boa_engine::JsValue::from( #ident ) )?;
        },
        quote! {
            if let Some(ref realm) = realm {
                realm.register_property(
                    boa_engine::js_string!( #name ),
                    boa_engine::JsValue::from( #ident ),
                    boa_engine::property::Attribute::all(),
                    context,
                )?;
            } else {
                context.register_global_property(
                    boa_engine::js_string!( #name ),
                    boa_engine::JsValue::from( #ident ),
                    boa_engine::property::Attribute::all(),
                )?;
            }
        },
    ))
}

fn fn_item(
    fn_: &mut ItemFn,
    renaming: RenameScheme,
) -> SpannedResult<(String, TokenStream2, TokenStream2)> {
    let ident = &fn_.sig.ident;
    let name = take_name_value_string(&mut fn_.attrs, "rename")?
        .unwrap_or_else(|| renaming.rename(ident.to_string()));

    if fn_.sig.asyncness.is_some() {
        error(&fn_.sig.asyncness, "Async methods are not supported.")?;
    }

    let fn_ = Function::from_sig(
        name.clone(),
        false,
        false,
        &mut fn_.attrs,
        &mut fn_.sig,
        None,
    )?;
    let fn_body = fn_.body();

    Ok((
        name.clone(),
        quote! {
            m.set_export(
                &boa_engine::js_string!( #name ),
                boa_engine::JsValue::from(
                    boa_engine::NativeFunction::from_fn_ptr( #fn_body )
                        .to_js_function(context.realm())
                ),
            )?;
        },
        quote! {
            let function = #fn_body;
            if let Some(ref realm) = realm {
                realm.register_property(
                    boa_engine::js_string!( #name ),
                    boa_engine::JsValue::from(
                        boa_engine::NativeFunction::from_fn_ptr( function )
                            .to_js_function(context.realm())
                    ),
                    boa_engine::property::Attribute::all(),
                    context,
                )?;
            } else {
                context.register_global_property(
                    boa_engine::js_string!( #name ),
                    boa_engine::JsValue::from(
                        boa_engine::NativeFunction::from_fn_ptr( function )
                            .to_js_function(context.realm())
                    ),
                    boa_engine::property::Attribute::all(),
                )?;
            }
        },
    ))
}

fn type_item(
    ty: &mut ItemType,
    renaming: RenameScheme,
) -> SpannedResult<(String, TokenStream2, TokenStream2)> {
    let ident = &ty.ident;
    let name = take_name_value_string(&mut ty.attrs, "rename")?
        .unwrap_or_else(|| renaming.rename(ident.to_string()));
    let path = ty.ty.as_ref();

    Ok((
        name.clone(),
        quote! {
            m.export_named_class::< #path >(&boa_engine::js_string!(#name), context)?;
        },
        quote! {
            if let Some(ref realm) = realm {
                let mut class_builder = boa_engine::class::ClassBuilder::new::<#path>(context);
                <#path as boa_engine::class::Class>::init(&mut class_builder)?;
                let class = class_builder.build();
                realm.register_class::<#path>(class);
            } else {
                context.register_global_class::<#path>()?;
            }
        },
    ))
}

pub(crate) fn module_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute arguments.
    let args = syn::parse_macro_input!(attr as ModuleArguments);

    // Parse the input.
    let mod_ = syn::parse_macro_input!(input as ItemMod);

    match module_impl_impl(args, mod_) {
        Ok(tokens) => tokens.into(),
        Err((span, msg)) => syn::Error::new(span, msg).to_compile_error().into(),
    }
}

// Allow too many lines as this is a giant match with local variables. The logic is still
// fairly straightforward.
#[allow(clippy::too_many_lines)]
fn module_impl_impl(_args: ModuleArguments, mut mod_: ItemMod) -> SpannedResult<TokenStream2> {
    let renaming = RenameScheme::from_named_attrs(&mut mod_.attrs, "rename_all")?
        .unwrap_or(RenameScheme::CamelCase);
    let class_renaming = RenameScheme::from_named_attrs(&mut mod_.attrs, "rename_all_class")?
        .unwrap_or(RenameScheme::PascalCase);

    // Iterate through all top-level content. If the module is empty, still
    // iterate to create an empty JS module.
    let mut original_module_decl = quote! {};
    let mut module_fn = quote! {};
    let mut global_fn = quote! {};
    let mut module_exports = quote! {};
    let mut generics = vec![];

    for item in mod_.content.map_or_else(Vec::new, |c| c.1).as_mut_slice() {
        // Check for skip attributes.
        match item {
            Item::Const(ItemConst { attrs, .. })
            | Item::Enum(ItemEnum { attrs, .. })
            | Item::ExternCrate(ItemExternCrate { attrs, .. })
            | Item::Fn(ItemFn { attrs, .. })
            | Item::ForeignMod(ItemForeignMod { attrs, .. })
            | Item::Impl(ItemImpl { attrs, .. })
            | Item::Macro(ItemMacro { attrs, .. })
            | Item::Mod(ItemMod { attrs, .. })
            | Item::Static(ItemStatic { attrs, .. })
            | Item::Struct(ItemStruct { attrs, .. })
            | Item::Trait(ItemTrait { attrs, .. })
            | Item::TraitAlias(ItemTraitAlias { attrs, .. })
            | Item::Type(ItemType { attrs, .. })
            | Item::Union(ItemUnion { attrs, .. })
            | Item::Use(ItemUse { attrs, .. }) => {
                if take_path_attr(attrs, "skip") {
                    original_module_decl = quote! {
                        #original_module_decl
                        #item
                    };
                    continue;
                }
            }
            _ => {}
        }

        let result = match item {
            Item::Const(c) => const_item(c, renaming),
            Item::Fn(f) => {
                generics.extend(f.sig.generics.params.iter().cloned());
                fn_item(f, renaming)
            }
            Item::Use(_) => {
                // Skip use statements. These are valid but ignored.
                original_module_decl = quote! {
                    #original_module_decl
                    #item
                };
                continue;
            }
            Item::Type(ty) => type_item(ty, class_renaming),
            _ => Err((
                item.span(),
                "Invalid boa_module top-level item.".to_string(),
            )),
        };
        let (export_name, export_decl, register_global) = result?;

        module_fn = quote! {
            #module_fn
            #export_decl
        };
        global_fn = quote! {
            #global_fn
            #register_global
        };
        module_exports = quote! {
            #module_exports
            boa_engine::js_string!( #export_name ),
        };
        original_module_decl = quote! {
            #original_module_decl

            #[allow(unused)]
            #item
        }
    }

    let debug = take_path_attr(&mut mod_.attrs, "debug");
    let vis = mod_.vis;
    let name = mod_.ident;
    let attrs = mod_.attrs;
    let safety = mod_.unsafety;

    let generics = quote! {
        <#(#generics),*>
    };

    let tokens = quote! {
        #(#attrs)*
        #vis #safety mod #name {
            #original_module_decl

            pub(super) fn boa_register #generics (
                realm: Option<boa_engine::realm::Realm>,
                context: &mut boa_engine::Context,
            ) -> boa_engine::JsResult<()> {
                #global_fn
                Ok(())
            }

            pub(super) fn boa_module #generics (
                realm: Option<boa_engine::realm::Realm>,
                context: &mut boa_engine::Context,
            ) -> boa_engine::Module {
                boa_engine::Module::synthetic(
                    &[ #module_exports ],
                    boa_engine::module::SyntheticModuleInitializer::from_copy_closure(
                        |m, context| {
                            #module_fn
                            Ok(())
                        }
                    ),
                    None,
                    realm,
                    context,
                )
            }
        }
    };

    #[allow(clippy::print_stderr)]
    if debug {
        eprintln!("---------\n{tokens}\n---------\n");
    }

    Ok(tokens)
}
