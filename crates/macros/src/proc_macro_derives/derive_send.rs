use proc_macro::TokenStream;

pub fn derive_send(input: syn::DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    quote::quote!(unsafe impl #impl_generics Send for #name #type_generics #where_clause {}).into()
}
