use proc_macro::TokenStream;

pub(crate) fn derive_extract_timestamp(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;

    let timestamp_field = if let syn::Data::Struct(data) = &input.data {
        data.fields.iter().find_map(|f| {
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
