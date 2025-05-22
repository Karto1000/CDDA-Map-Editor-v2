use proc_macro::TokenStream as PrimTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;

fn cdda_entry_impl(tokens: TokenStream) -> TokenStream {
    let struct_ast = syn::parse2::<syn::ItemStruct>(tokens.clone()).unwrap();

    let struct_ident = struct_ast.ident.clone();
    let intermediate_struct_name = Ident::new(
        format!("{}Intermediate", struct_ast.ident).as_str(),
        struct_ast.ident.span(),
    );

    let mut predefined_fields = HashSet::new();
    predefined_fields.insert("id".to_string());
    predefined_fields.insert("flags".to_string());
    predefined_fields.insert("copy_from".to_string());
    predefined_fields.insert("extend".to_string());
    predefined_fields.insert("delete".to_string());

    let extra_fields = struct_ast
        .fields
        .clone()
        .into_iter()
        .filter(|f| {
            let field_ident = f.ident.clone().unwrap();
            !predefined_fields.contains(&field_ident.to_string())
        })
        .collect::<Vec<_>>();

    let extra_field_idents = extra_fields
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    let impl_merge = {
        let mut extra_optional_fields = vec![];
        let mut extra_required_fields = vec![];

        for f in extra_fields.iter() {
            let field_ident = f.ident.clone().unwrap();

            if let syn::Type::Path(syn::TypePath { path, .. }) = &f.ty {
                let ident = path.segments.first().unwrap().ident.to_string();

                if ident == "Option" {
                    extra_optional_fields.push(field_ident);
                    continue;
                }

                extra_required_fields.push(field_ident);
            };
        }

        let extra_optional_fields_concat = match extra_optional_fields.len() {
            0 => None,
            _ => Some(
                quote! { #(#extra_optional_fields: override_.#extra_optional_fields.clone().or(base.#extra_optional_fields.clone())),* },
            ),
        };

        let extra_required_fields_concat = match extra_required_fields.len() {
            0 => None,
            _ => Some(
                quote! { #(#extra_required_fields: override_.#extra_required_fields.clone()),* },
            ),
        };

        let full_segment = match (
            extra_optional_fields_concat,
            extra_required_fields_concat,
        ) {
            (
                Some(extra_optional_fields_concat),
                Some(extra_required_fields_concat),
            ) => {
                quote! {
                    #extra_optional_fields_concat,
                    #extra_required_fields_concat
                }
            },
            (Some(extra_optional_fields_concat), None) => {
                quote! {
                    #extra_optional_fields_concat
                }
            },
            (None, Some(extra_required_fields_concat)) => {
                quote! {
                    #extra_required_fields_concat
                }
            },
            (None, None) => {
                quote! {}
            },
        };

        quote! {
            fn merge(base: &Self, override_: &Self) -> Self {
                Self {
                    id: base.id.clone(),
                    flags: base.flags.clone(),
                    copy_from: override_.copy_from.clone(),
                    extend: override_.extend.clone(),
                    delete: override_.delete.clone(),
                    #full_segment
                }
            }
        }
    };

    quote! {
        #tokens

        #[derive(serde::Deserialize, Debug, Clone)]
        pub struct #intermediate_struct_name {
            #[serde(alias = "id", alias = "abstract")]
            pub id: cdda_lib::types::MeabyVec<CDDAIdentifier>,

            #[serde(default)]
            pub flags: Vec<String>,

            #[serde(rename = "copy-from")]
            pub copy_from: Option<cdda_lib::types::CDDAIdentifier>,

            pub extend: Option<cdda_lib::types::CDDAExtendOp>,
            pub delete: Option<cdda_lib::types::CDDADeleteOp>,

            #(#extra_fields),*
        }

        impl cdda_lib::types::ImportCDDAObject for #intermediate_struct_name {
            #impl_merge

            fn copy_from(&self) -> Option<&cdda_lib::types::CDDAIdentifier> {
                self.copy_from.as_ref()
            }

            fn extend(&self) -> Option<&cdda_lib::types::CDDAExtendOp> {
                self.extend.as_ref()
            }

            fn delete(&self) -> Option<&cdda_lib::types::CDDADeleteOp> {
                self.delete.as_ref()
            }

            fn flags(&self) -> &Vec<String> {
                self.flags.as_ref()
            }

            fn set_flags(&mut self, flags: Vec<String>) {
                self.flags = flags;
            }
        }

        impl Into<Vec<#struct_ident>> for #intermediate_struct_name {
            fn into(self) -> Vec<#struct_ident> {
                let mut all_vals = vec![];

                for ident in self.id.clone().into_vec() {
                    all_vals.push(#struct_ident {
                        id: ident,
                        flags: self.flags.clone(),
                        #(#extra_field_idents: self.#extra_field_idents.clone()),*
                    })
                }

                all_vals
            }
        }

        impl Into<#struct_ident> for #intermediate_struct_name {
            fn into(self) -> #struct_ident {
                let vec = self.id.clone().into_vec();

                assert!(vec.len() == 1, "Expected exactly one value in id field {:?}", self.id);

                let id = match vec.first() {
                    None => panic!(),
                    Some(val) => {
                        val.clone()
                    }
                };

                #struct_ident {
                    id: id.clone(),
                    flags: self.flags,
                    #(#extra_field_idents: self.#extra_field_idents),*
                }
            }
        }
    }
}

#[proc_macro_attribute]
pub fn cdda_entry(
    _attr: PrimTokenStream,
    item: PrimTokenStream,
) -> PrimTokenStream {
    cdda_entry_impl(item.into()).into()
}

/*

#[generate_intermediate]
struct TestObject {
    pub id: CDDAIdentifier,
    pub name: Option<String>,
    ...
}

------
#[generate_intermediate]
------
pub struct TestObjectIntermediate {
    pub id: IdOrAbstract<T>
                         ^
               id: CDDAIdentifier

    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier

    pub flags: Option<Vec<String>>
}

impl Into<TestObject> for TestObjectIntermediate {
    fn into(self) -> TestObject {
        let (id, is_abstract) = match self.identifier {
            IdOrAbstract::Id(id) => (id, false),
            IdOrAbstract::Abstract(abs) => (abs, true),
        };
        TestObject {
            id: self.id.into(),
            name: self.name,
            ...
        }
    }
}

impl TestObject {
    pub fn calculate_copy(
        &self,
        cdda_data: &DeserializedCDDAData
    ) -> TestObject {
        ..
    }
}

 */
