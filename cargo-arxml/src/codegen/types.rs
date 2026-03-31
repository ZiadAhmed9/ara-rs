use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::error::CargoArxmlError;
use crate::parser::ir::{ArxmlProject, DataType, DataTypeKind, PrimitiveType};

use super::snake_case;

/// Generate a `types.rs` file containing all data type definitions extracted
/// from the ARXML model.
pub fn generate_types(project: &ArxmlProject) -> Result<String, CargoArxmlError> {
    let mut items: Vec<TokenStream> = Vec::new();

    items.push(quote! {
        use ara_com::transport::{AraSerialize, AraDeserialize};
        use ara_com::error::AraComError;
    });

    for dt in &project.data_types {
        let ts = generate_type(dt, project).map_err(|e| CargoArxmlError::CodeGen {
            message: format!("type '{}': {}", dt.name, e),
        })?;
        items.push(ts);
    }

    let combined: TokenStream = items.into_iter().collect();

    let file: syn::File = syn::parse2(combined).map_err(|e| CargoArxmlError::CodeGen {
        message: format!("failed to parse generated types: {e}"),
    })?;

    Ok(prettyplease::unparse(&file))
}

// ---------------------------------------------------------------------------
// Per-type generation
// ---------------------------------------------------------------------------

fn generate_type(dt: &DataType, project: &ArxmlProject) -> Result<TokenStream, String> {
    let type_name = Ident::new(&dt.name, Span::call_site());
    let doc = dt
        .description
        .as_deref()
        .map(|d| quote! { #[doc = #d] })
        .unwrap_or_default();

    match &dt.kind {
        DataTypeKind::Primitive(prim) => {
            let rust_type = primitive_to_tokens(prim);
            Ok(quote! {
                #doc
                pub type #type_name = #rust_type;
            })
        }

        DataTypeKind::TypeReference { target_ref } => {
            let target = resolve_type_name(target_ref, project);
            let target_ts: TokenStream = target
                .parse()
                .map_err(|e| format!("invalid type ref '{target_ref}': {e}"))?;
            Ok(quote! {
                #doc
                pub type #type_name = #target_ts;
            })
        }

        DataTypeKind::String { .. } => Ok(quote! {
            #doc
            pub type #type_name = String;
        }),

        DataTypeKind::Array {
            element_type_ref,
            size,
        } => {
            let elem_name = resolve_type_name(element_type_ref, project);
            let elem_ts: TokenStream = elem_name
                .parse()
                .map_err(|e| format!("invalid element type ref '{element_type_ref}': {e}"))?;
            match size {
                Some(n) => {
                    let lit = Literal::usize_unsuffixed(*n);
                    Ok(quote! {
                        #doc
                        pub type #type_name = [#elem_ts; #lit];
                    })
                }
                None => Ok(quote! {
                    #doc
                    pub type #type_name = Vec<#elem_ts>;
                }),
            }
        }

        DataTypeKind::Vector { element_type_ref } => {
            let elem_name = resolve_type_name(element_type_ref, project);
            let elem_ts: TokenStream = elem_name
                .parse()
                .map_err(|e| format!("invalid element type ref '{element_type_ref}': {e}"))?;
            Ok(quote! {
                #doc
                pub type #type_name = Vec<#elem_ts>;
            })
        }

        DataTypeKind::Structure { fields } => {
            let field_defs: Vec<TokenStream> = fields
                .iter()
                .map(|f| {
                    let field_name = Ident::new(&snake_case(&f.name), Span::call_site());
                    let field_type_name = resolve_type_name(&f.type_ref, project);
                    let field_type: TokenStream =
                        field_type_name.parse().unwrap_or_else(|_| quote! { () });
                    quote! { pub #field_name: #field_type, }
                })
                .collect();

            // Serialization: each field in order
            let serialize_fields: Vec<TokenStream> = fields
                .iter()
                .map(|f| {
                    let field_name = Ident::new(&snake_case(&f.name), Span::call_site());
                    quote! { self.#field_name.ara_serialize(buf)?; }
                })
                .collect();

            // serialized_size: sum of all field sizes
            let size_fields: Vec<TokenStream> = if fields.is_empty() {
                vec![quote! { 0 }]
            } else {
                fields
                    .iter()
                    .map(|f| {
                        let field_name = Ident::new(&snake_case(&f.name), Span::call_site());
                        quote! { self.#field_name.serialized_size() }
                    })
                    .collect()
            };

            // Deserialization: each field in order, advancing offset
            let field_count = fields.len();
            let deser_fields: Vec<TokenStream> = fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let field_name = Ident::new(&snake_case(&f.name), Span::call_site());
                    let field_type_name = resolve_type_name(&f.type_ref, project);
                    let field_type: TokenStream = field_type_name
                        .parse()
                        .unwrap_or_else(|_| quote! { () });
                    if i + 1 < field_count {
                        quote! {
                            let #field_name = <#field_type as AraDeserialize>::ara_deserialize(&buf[offset..])?;
                            offset += #field_name.serialized_size();
                        }
                    } else {
                        quote! {
                            let #field_name = <#field_type as AraDeserialize>::ara_deserialize(&buf[offset..])?;
                        }
                    }
                })
                .collect();

            let field_names: Vec<Ident> = fields
                .iter()
                .map(|f| Ident::new(&snake_case(&f.name), Span::call_site()))
                .collect();

            Ok(quote! {
                #doc
                #[derive(Debug, Clone, PartialEq)]
                pub struct #type_name {
                    #(#field_defs)*
                }

                impl AraSerialize for #type_name {
                    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
                        #(#serialize_fields)*
                        Ok(())
                    }

                    fn serialized_size(&self) -> usize {
                        #(#size_fields)+*
                    }
                }

                impl AraDeserialize for #type_name {
                    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
                        #[allow(unused_mut)]
                        let mut offset = 0usize;
                        #(#deser_fields)*
                        Ok(Self {
                            #(#field_names,)*
                        })
                    }
                }
            })
        }

        DataTypeKind::Enumeration { variants } => {
            let variant_defs: Vec<TokenStream> = variants
                .iter()
                .map(|v| {
                    let vname = Ident::new(&v.name, Span::call_site());
                    let vval = Literal::i64_unsuffixed(v.value);
                    quote! { #vname = #vval, }
                })
                .collect();

            let first_variant = if let Some(v) = variants.first() {
                let vname = Ident::new(&v.name, Span::call_site());
                quote! { #type_name::#vname }
            } else {
                quote! { compile_error!("enum has no variants") }
            };

            let match_arms: Vec<TokenStream> = variants
                .iter()
                .map(|v| {
                    let vname = Ident::new(&v.name, Span::call_site());
                    let vval = Literal::i64_unsuffixed(v.value);
                    quote! { #vval => Ok(#type_name::#vname), }
                })
                .collect();

            Ok(quote! {
                #doc
                #[derive(Debug, Clone, Copy, PartialEq)]
                #[repr(i64)]
                pub enum #type_name {
                    #(#variant_defs)*
                }

                impl AraSerialize for #type_name {
                    fn ara_serialize(&self, buf: &mut Vec<u8>) -> Result<(), AraComError> {
                        let discriminant = *self as i64;
                        discriminant.ara_serialize(buf)
                    }

                    fn serialized_size(&self) -> usize {
                        8
                    }
                }

                impl AraDeserialize for #type_name {
                    fn ara_deserialize(buf: &[u8]) -> Result<Self, AraComError> {
                        let discriminant = i64::ara_deserialize(buf)?;
                        match discriminant {
                            #(#match_arms)*
                            _ => Err(AraComError::Deserialization {
                                message: format!("unknown discriminant {} for {}", discriminant, stringify!(#type_name)),
                            }),
                        }
                    }
                }

                impl Default for #type_name {
                    fn default() -> Self {
                        #first_variant
                    }
                }
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a [`PrimitiveType`] to its `TokenStream` Rust equivalent.
fn primitive_to_tokens(prim: &PrimitiveType) -> TokenStream {
    match prim {
        PrimitiveType::Bool => quote! { bool },
        PrimitiveType::U8 => quote! { u8 },
        PrimitiveType::U16 => quote! { u16 },
        PrimitiveType::U32 => quote! { u32 },
        PrimitiveType::U64 => quote! { u64 },
        PrimitiveType::I8 => quote! { i8 },
        PrimitiveType::I16 => quote! { i16 },
        PrimitiveType::I32 => quote! { i32 },
        PrimitiveType::I64 => quote! { i64 },
        PrimitiveType::F32 => quote! { f32 },
        PrimitiveType::F64 => quote! { f64 },
    }
}

/// Map an AUTOSAR type-ref path to a Rust type name string.
///
/// Rules (in order):
/// 1. Well-known AUTOSAR base-type paths → Rust primitive name.
/// 2. The last segment of the path, if it matches an IR data type name → that name.
/// 3. Fall back to the last segment as-is.
pub(crate) fn resolve_type_name(type_ref: &str, project: &ArxmlProject) -> String {
    // 0. Empty or missing type refs fall back to unit type
    if type_ref.is_empty() {
        return "()".to_string();
    }

    // 1. Well-known primitives
    let last = type_ref.split('/').next_back().unwrap_or(type_ref);
    let primitive = match last.to_lowercase().as_str() {
        "boolean" | "bool" => Some("bool"),
        "uint8" | "u8" => Some("u8"),
        "uint16" | "u16" => Some("u16"),
        "uint32" | "u32" => Some("u32"),
        "uint64" | "u64" => Some("u64"),
        "sint8" | "int8" | "i8" => Some("i8"),
        "sint16" | "int16" | "i16" => Some("i16"),
        "sint32" | "int32" | "i32" => Some("i32"),
        "sint64" | "int64" | "i64" => Some("i64"),
        "float32" | "f32" => Some("f32"),
        "float64" | "f64" => Some("f64"),
        _ => None,
    };
    if let Some(p) = primitive {
        return p.to_string();
    }

    // 2. Check against IR data type names
    for dt in &project.data_types {
        if dt.path == type_ref || dt.name == last {
            return dt.name.clone();
        }
    }

    // 3. Fall back to last segment
    last.to_string()
}
