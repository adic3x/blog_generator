#[proc_macro_attribute]
pub fn template(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(input as syn::DeriveInput);

    input.attrs.iter_mut().for_each(|attr| {
        if attr.path().is_ident("template") {
            let mut path_str = String::new();
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") {
                    path_str = meta.value()?.parse::<syn::LitStr>()?.value();
                }
                Ok(())
            }).expect("Can't parse attributes");

            let minified_str = std::fs::read_to_string(&path_str)
                .expect("Could not find template file")
                .lines()
                .map(str::trim)
                .collect::<String>();

            *attr = syn::parse_quote! { #[template(source = #minified_str, ext = "html")] };
        }
    });

    proc_macro::TokenStream::from(quote::quote! { #input })
}