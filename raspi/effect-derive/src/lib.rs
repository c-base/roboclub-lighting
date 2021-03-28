#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Schema, attributes(schema))]]
pub fn derive_schema(input: TokenStream) -> TokenStream {


	let expanded = quote! {
		impl #impl_generics crate::effects::schema::Schema for #name #ty_generics #where_clause {
			fn schema(&self) -> serde_json::Value {
				let map

			}

      fn create_buffers(&self, renderer: &Renderer, indices: &[u16]) -> (VAO, Vec<VBO>, VBO) {
        #(#buffers)*

        let index_buffer = renderer.create_index_buffer(DataType::U16, 3, indices);

        let mut vao = renderer.create_vertex_array();
        vao
          #(#buffer_attrs)*
          .index_buffer(&index_buffer);

        (vao, vec![#(#buffer_names),*], index_buffer)
      }

      fn vertex_count(&self) -> usize {
        self.#first_name.len() / #first_size
      }
    }
  };

	TokenStream::from(expanded)
}
