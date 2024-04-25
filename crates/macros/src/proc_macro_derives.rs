use proc_macro::TokenStream;
use syn::TypeParam;

pub fn derive_send(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    quote::quote!(unsafe impl #impl_generics Send for #name #type_generics #where_clause {}).into()
}

pub fn derive_sync(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    quote::quote!(unsafe impl #impl_generics Sync for #name #type_generics #where_clause {}).into()
}

pub fn derive_unpin(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    quote::quote!(impl #impl_generics Unpin for #name #type_generics #where_clause {}).into()
}

pub fn derive_deep_clone(mut input: syn::DeriveInput) -> TokenStream {
    for param in &mut input.generics.params {
        if let syn::GenericParam::Type(TypeParam { bounds, .. }) = param {
            bounds.push(syn::parse_quote!(DeepClone));
        }
    }

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    match &input.data {
        syn::Data::Struct(data) => {
            let fields = data.fields.iter().enumerate().map(|(i, field)| {
                if let Some(name) = &field.ident {
                    quote::quote!(#name: <_ as DeepClone>::deep_clone(&self.#name))
                } else {
                    let index = syn::Index::from(i);
                    quote::quote!(#index: <_ as DeepClone>::deep_clone(&self.#index))
                }
            });
            quote::quote!(
                impl #impl_generics DeepClone for #name #type_generics #where_clause {
                    #[inline(always)]
                    fn deep_clone(&self) -> Self {
                        Self {
                            #(#fields),*
                        }
                    }
                }
            )
            .into()
        }
        syn::Data::Enum(data) => {
            let variants = data.variants.iter().map(|variant| {
                let name = &variant.ident;
                let names = variant.fields.iter().enumerate().map(|(i, field)| {
                    if let Some(name) = &field.ident {
                        quote::quote!(#name)
                    } else {
                        let index = syn::Index::from(i);
                        let id =
                            syn::Ident::new(&format!("field{}", i), proc_macro2::Span::call_site());
                        quote::quote!(#index: #id)
                    }
                });
                let fields = variant.fields.iter().enumerate().map(|(i, field)| {
                    if let Some(name) = &field.ident {
                        quote::quote!(#name: <_ as DeepClone>::deep_clone(&self.#name))
                    } else {
                        let index = syn::Index::from(i);
                        let id =
                            syn::Ident::new(&format!("field{}", i), proc_macro2::Span::call_site());
                        quote::quote!(#index: <_ as DeepClone>::deep_clone(#id))
                    }
                });
                quote::quote!(Self::#name { #(#names),* } => Self::#name { #(#fields),* })
            });
            quote::quote!(
                impl #impl_generics DeepClone for #name #type_generics #where_clause {
                    #[inline(always)]
                    fn deep_clone(&self) -> Self {
                        match self {
                            #(#variants),*
                        }
                    }
                }
            )
            .into()
        }
        syn::Data::Union(_) => unreachable!(),
    }
}

pub(crate) fn derive_extract_timestamp(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;

    let timestamp_field = if let syn::Data::Struct(data) = &input.data {
        data.fields
            .iter()
            .find_map(|f| {
                f.attrs
                    .iter()
                    .find_map(|attr| attr.path().is_ident("timestamp").then_some(f))
            })
    } else {
        None
    };

    if let Some(field) = timestamp_field {
        let field_name = &field.ident;
        quote::quote! {
            impl #impl_generics ::runtime::traits::Timestamp for #name #type_generics #where_clause {
                #[inline(always)]
                fn timestamp(&self) -> Time {
                    Time::from_milliseconds(self.#field_name as i128)
                }
            }
        }
        .into()
    } else {
        quote::quote!().into()
    }
}

pub(crate) fn derive_new(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = input.ident;
    let vis = input.vis;

    let fields = if let syn::Data::Struct(data) = input.data {
        if let syn::Fields::Named(fields) = data.fields {
            fields.named
        } else {
            panic!("#[derive(New)] only supports structs with named fields");
        }
    } else {
        panic!("#[derive(New)] only supports structs");
    };

    let params = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote::quote! { #name: #ty }
    });

    let init_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote::quote! { #name: #name }
    });

    let gen = quote::quote! {
        impl #impl_generics #name #type_generics #where_clause {
            #[inline(always)]
            #vis fn new(#(#params),*) -> Self {
                Self {
                    #(#init_fields),*
                }
            }
        }
    };

    gen.into()
}
