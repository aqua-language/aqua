use proc_macro::TokenStream;

pub fn derive_sync(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    quote::quote!(unsafe impl #impl_generics Sync for #name #type_generics #where_clause {}).into()
}
