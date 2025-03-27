use proc_macro::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;

pub fn enumset(item: DeriveInput) -> TokenStream {
    let name = &item.ident;

    let data_enum = match &item.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("bitmask can only be applied to enums"),
    };

    let bit_values = data_enum.variants.iter().enumerate().map(|(i, variant)| {
        let variant_name = &variant.ident;
        if let Fields::Unit = &variant.fields {
            quote! {
                pub const #variant_name: Self = Self(1 << #i);
            }
        } else {
            panic!("Bitmask enum variants must be unit variants")
        }
    });

    let expanded = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct #name(u128);

        #[allow(non_upper_case_globals)]
        impl #name {
            #(#bit_values)*

            pub const fn empty() -> Self {
                Self(0)
            }

            pub const fn bits(&self) -> u128 {
                self.0
            }

            pub const fn contains(&self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            pub const fn from(bits: u128) -> Self {
                Self(bits)
            }

            pub const fn or(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl std::ops::BitOr for #name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
    };

    expanded.into()
}
