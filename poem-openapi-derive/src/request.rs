use std::str::FromStr;

use darling::{
    ast::{Data, Fields},
    util::Ignored,
    FromDeriveInput, FromVariant,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput, Error, Generics, Type};

use crate::{
    error::GeneratorResult,
    utils::{get_crate_name, get_description, optional_literal},
};

#[derive(FromVariant)]
#[darling(attributes(oai), forward_attrs(doc))]
struct RequestItem {
    ident: Ident,
    fields: Fields<Type>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(oai), forward_attrs(doc))]
struct RequestArgs {
    ident: Ident,
    attrs: Vec<Attribute>,
    generics: Generics,
    data: Data<RequestItem, Ignored>,

    #[darling(default)]
    internal: bool,
}

pub(crate) fn generate(args: DeriveInput) -> GeneratorResult<TokenStream> {
    let args: RequestArgs = RequestArgs::from_derive_input(&args)?;
    let crate_name = get_crate_name(args.internal);
    let (impl_generics, ty_generics, where_clause) = args.generics.split_for_impl();
    let ident = &args.ident;
    let e = match &args.data {
        Data::Enum(e) => e,
        _ => {
            return Err(Error::new_spanned(ident, "Request can only be applied to an enum.").into())
        }
    };
    let description = get_description(&args.attrs)?;
    let description = optional_literal(&description);

    let mut from_requests = Vec::new();
    let mut content = Vec::new();
    let mut schemas = Vec::new();

    let impl_generics = {
        let mut s = quote!(#impl_generics).to_string();
        match s.find('<') {
            Some(pos) => {
                s.insert_str(pos + 1, "'__request,");
                TokenStream::from_str(&s).unwrap()
            }
            _ => quote!(<'__request>),
        }
    };

    for variant in e {
        let item_ident = &variant.ident;

        match variant.fields.len() {
            1 => {
                // Item(payload)
                let payload_ty = &variant.fields.fields[0];
                from_requests.push(quote! {
                    ::std::option::Option::Some(<#payload_ty as #crate_name::payload::Payload>::CONTENT_TYPE) => {
                        ::std::result::Result::Ok(#ident::#item_ident(
                            <#payload_ty as #crate_name::payload::ParsePayload>::from_request(request, body).await?
                        ))
                    }
                });
                content.push(quote! {
                    #crate_name::registry::MetaMediaType {
                        content_type: <#payload_ty as #crate_name::payload::Payload>::CONTENT_TYPE,
                        schema: <#payload_ty as #crate_name::payload::Payload>::schema_ref(),
                    }
                });
                schemas.push(payload_ty);
            }
            _ => {
                return Err(
                    Error::new_spanned(&variant.ident, "Incorrect request definition.").into(),
                )
            }
        }
    }

    let expanded = {
        quote! {
            #[#crate_name::__private::poem::async_trait]
            impl #impl_generics #crate_name::ApiExtractor<'__request> for #ident #ty_generics #where_clause {
                const TYPE: #crate_name::ApiExtractorType = #crate_name::ApiExtractorType::RequestObject;

                type ParamType = ();
                type ParamRawType = ();

                fn register(registry: &mut #crate_name::registry::Registry) {
                    #(<#schemas as #crate_name::payload::Payload>::register(registry);)*
                }

                fn request_meta() -> ::std::option::Option<#crate_name::registry::MetaRequest> {
                    ::std::option::Option::Some(#crate_name::registry::MetaRequest {
                        description: #description,
                        content: ::std::vec![#(#content),*],
                        required: true,
                    })
                }

                async fn from_request(
                    request: &'__request #crate_name::__private::poem::Request,
                    body: &mut #crate_name::__private::poem::RequestBody,
                    _param_opts: #crate_name::ExtractParamOptions<Self::ParamType>,
                ) -> ::std::result::Result<Self, #crate_name::ParseRequestError> {
                    let content_type = request.content_type();
                    match content_type {
                        #(#from_requests)*
                        ::std::option::Option::Some(content_type) => ::std::result::Result::Err(#crate_name::ParseRequestError::ContentTypeNotSupported {
                            content_type: ::std::string::ToString::to_string(content_type),
                        }),
                        ::std::option::Option::None => ::std::result::Result::Err(#crate_name::ParseRequestError::ExpectContentType),
                    }
                }
            }
        }
    };

    Ok(expanded)
}
