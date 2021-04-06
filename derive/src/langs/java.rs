use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Attribute, Fields, FieldsNamed, FnArg, Ident, ImplItem, ImplItemMethod, Item, ItemFn, Pat, PatIdent, PatType, Token, TraitItem, TraitItemMethod, TypeReference, ItemUse, Stmt};

use super::*;
use crate::types::*;
use syn::token::Token;

#[derive(Debug)]
pub struct Java;

impl Lang for Java {
    type Error = JavaError;

    fn expose_fn(function: &mut ItemFn, mod_path: &Vec<Ident>) -> Result<Ident, Self::Error> {

        if mod_path.is_empty() {
            return Err(JavaError::NakedFunction);
        }

        let ident = &function.sig.ident;
        let java_path = mod_path.to_vec();

        let (mut args, input_conversion) = Self::convert_fn_args(function.sig.inputs.clone())?;
        
        args.insert(0, parse_quote!(class: JClass));
        args.insert(0, parse_quote!(env: JNIEnv));
        
        let ExpandedReturn {
            ret,
            extra_args,
            conv: output_conversion,
        } = Return(function.sig.output.clone()).expand(
            &format_ident!("__output"),
            &format_ident!("__ptr_out"),
            Self::convert_output,
        )?;
        args.extend(extra_args);

        let block = &function.block;

        //let ident_str = ident.to_string();
        *function = parse_quote! {
            #[no_mangle]
            pub extern "system" fn #ident(#args) #ret {
                
                use crate::mapping::{MapTo, MapFrom};
                use crate::langs::*;

                #input_conversion

                let mut block_closure = move || { #block };
                let __output = block_closure();

                #output_conversion
            }
        };

        Ok(function.sig.ident.clone())
    }

    fn expose_mod(
        module: &mut ItemMod,
        _mod_path: &Vec<Ident>,
        _sub_items: Vec<ModuleItem>,
    ) -> Result<Ident, Self::Error> {

        let ident = &module.ident;
        let content = &mut module.content.as_mut().expect("Empty module").1;
        let mut content_tokens = TokenStream2::default();
        content_tokens.append_all(content);
        
        *module = parse_quote! {
            pub mod #ident {
                use jni::JNIEnv;
                use jni::objects::{JValue};
                use jni::sys::{jint};   
            
                #content_tokens
            }
        };

        Ok(module.ident.clone())
    }

    fn expose_struct(
        structure: &mut ItemStruct,
        opts: Punctuated<ExposeStructOpts, Token![,]>,
        mod_path: &Vec<Ident>,
        extra: &mut Vec<Item>,
    ) -> Result<Ident, Self::Error> {
        todo!()
    }

    fn expose_impl(
        implementation: &mut ItemImpl,
        mod_path: &Vec<Ident>,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn expose_trait(
        tr: &mut ItemTrait,
        mod_path: &Vec<Ident>,
        extra: &mut Vec<Item>,
    ) -> Result<Ident, Self::Error> {
        todo!()
    }

    fn expose_getter(
        structure: &Ident,
        field: &mut Field,
        _is_opaque: bool,
        _is_simple: bool,
        impl_block: &mut ItemImpl,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn expose_setter(
        structure: &Ident,
        field: &mut Field,
        _is_opaque: bool,
        _is_simple: bool,
        impl_block: &mut ItemImpl,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn wrap_field_type(ty: Type) -> Result<Type, Self::Error> {
        todo!()
    }

    fn convert_input(input: Type) -> Result<Input, Self::Error> {
        if input == parse_quote!(u32) {
            // TODO need to use jlong if value > java Integer.MAX_VALUE
            Ok(Input::new_map_from(input, vec![parse_quote!(jint)]))
        } else {
            Ok(Input::new_unchanged(input))
        }
    }

    fn convert_output(output: Type) -> Result<Output, Self::Error> {
        if output == parse_quote!(u32) {
            // TODO need to use jlong if value > java Integer.MAX_VALUE
            Ok(Output::new_map_to(output, vec![parse_quote!(jint)]))
        } else {
            Ok(Output::new_unchanged(output))
        }
    }
}

#[derive(Debug)]
pub enum JavaError {
    NakedFunction,

    Lang(LangError),
}

impl fmt::Display for JavaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for JavaError {}

impl From<LangError> for JavaError {
    fn from(e: LangError) -> Self {
        JavaError::Lang(e)
    }
}
