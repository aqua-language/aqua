use proc_macro::TokenStream;

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
