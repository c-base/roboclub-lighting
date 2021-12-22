#![feature(proc_macro_diagnostic)]
extern crate proc_macro;

use std::collections::HashMap;

use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::{
	parse_macro_input,
	spanned::Spanned,
	Attribute,
	Data,
	DeriveInput,
	Expr,
	ExprLit,
	Ident,
	Lit,
	LitBool,
	Meta,
	NestedMeta,
	Path,
};

#[proc_macro_derive(Schema, attributes(schema))]
#[proc_macro_error]
pub fn derive_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let expanded = expand(input);

	proc_macro::TokenStream::from(expanded)
}

fn expand(input: DeriveInput) -> TokenStream {
	let name = input.ident;

	let fields = parse_fields(&input.attrs);
	let (variants, num_vars) = collect_variants(&input.data, &fields);

	//	dbg!(input.data);

	let mut methods = Vec::new();
	let empty = HashMap::new();

	for (method_name, (method_name_ident, return_type)) in fields {
		let variants = variants.get(&method_name).unwrap_or(&empty);

		let mut variants_out = Vec::new();

		for (variant_name, variant_expr) in variants {
			if return_type.is_ident("String") {
				variants_out.push(quote!(#variant_name => #variant_expr.to_string()));
			} else {
				variants_out.push(quote!(#variant_name => #variant_expr));
			}
		}

		if variants_out.len() < num_vars {
			variants_out.push(quote!(_ => Default::default()));
		}

		methods.push(quote! {
			pub fn #method_name_ident(&self) -> #return_type {
				use #name::*;

				match self {
					#(#variants_out),*
				}
			}
		});
	}

	let expanded = quote! {
		impl #name {
			#(#methods)*
		}
	};

	expanded
}

fn collect_variants(
	data: &Data,
	fields: &HashMap<String, (Ident, Path)>,
) -> (HashMap<String, HashMap<Ident, Expr>>, usize) {
	let data_enum = match data {
		Data::Enum(e) => e,
		Data::Struct(s) => abort!(s.struct_token.span(), "only enums are supported"),
		Data::Union(u) => abort!(u.union_token.span(), "only enums are supported"),
	};

	let mut variants = HashMap::new();

	for variant in data_enum.variants.iter() {
		let attrs = parse_attributes(&variant.attrs);

		for (name, (_, lit)) in attrs {
			let parsed_lit = match &lit {
				Lit::Str(string) => {
					if fields.get(&name).unwrap().1.is_ident("String") {
						Expr::Lit(ExprLit { attrs: vec![], lit })
					} else {
						match string.parse::<Expr>() {
							Ok(lit) => lit,
							Err(error) => abort!(lit.span(), "Fd {}", error),
						}
					}
				}
				Lit::Bool(_) => Expr::Lit(ExprLit { attrs: vec![], lit }),
				_ => abort!(lit.span(), "Only strings allowed"),
			};

			let field_entry = variants.entry(name).or_insert_with(|| HashMap::new());

			field_entry.insert(variant.ident.clone(), parsed_lit);
		}
	}

	(variants, data_enum.variants.len())
}

fn parse_fields(attrs: &Vec<Attribute>) -> HashMap<String, (Ident, Path)> {
	let attributes = parse_attributes(attrs);

	let mut fields = HashMap::new();

	for (ident_str, (ident, literal)) in attributes {
		let value = match &literal {
			Lit::Str(string) => string.parse::<Path>(),
			_ => abort!(literal.span(), "Value should be a string"),
		};

		let value = match value {
			Ok(p) => p,
			_ => abort!(literal.span(), "Value should be a valid type"),
		};

		fields.insert(ident_str, (ident, value));
	}

	fields
}

fn parse_attributes(attrs: &Vec<Attribute>) -> HashMap<String, (Ident, Lit)> {
	let mut attributes = HashMap::new();

	for attr in attrs.iter() {
		let meta = match attr.parse_meta() {
			Ok(meta) => meta,
			Err(error) => abort!(attr.span(), "failed parsing as Meta: {}", error),
		};

		let list = match meta {
			Meta::List(list) if list.path.is_ident("enum_values") => list,
			_ => continue,
		};

		for entry in list.nested {
			let (path, value) = match entry {
				NestedMeta::Meta(Meta::NameValue(meta)) => (meta.path, meta.lit),
				NestedMeta::Meta(Meta::Path(meta)) => (
					meta.clone(),
					Lit::Bool(LitBool {
						value: true,
						span:  meta.span().clone(),
					}),
				),
				_ => abort!(entry.span(), "Inner should be name value meta"),
			};

			let ident = path
				.get_ident()
				.expect("Name value key should be ident")
				.clone();
			let ident_str = ident.to_string();

			attributes.insert(ident_str, (ident, value));
		}
	}

	attributes
}
