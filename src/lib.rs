mod gen_structs;

#[proc_macro]
pub fn gen_structs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    gen_structs::gen_structs(input)
}
