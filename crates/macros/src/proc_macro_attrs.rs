pub mod enumset;

use proc_macro as pm;

pub(crate) fn data(item: syn::DeriveInput) -> pm::TokenStream {
    quote::quote! {
        #[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New)]
        #[serde(crate = "runtime::prelude::serde")]
        // #[serde(bound(deserialize = ""))]
        #item
    }
    .into()
}
