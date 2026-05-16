use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub(crate) fn debug_unreachable(input: TokenStream) -> TokenStream {
    expand(input.into()).into()
}

fn expand(input: TokenStream2) -> TokenStream2 {
    if input.is_empty() {
        quote! { debug_assert!(false, "reached unreachable code") }
    } else {
        quote! { debug_assert!(false, #input) }
    }
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;

    #[test]
    fn should_emit_default_message_with_no_args() {
        let output = expand(quote!());
        assert_eq!(
            output.to_string(),
            quote! { debug_assert!(false, "reached unreachable code") }.to_string()
        );
    }

    #[test]
    fn should_emit_passthrough_with_literal_message() {
        let output = expand(quote!("custom message"));
        assert_eq!(
            output.to_string(),
            quote! { debug_assert!(false, "custom message") }.to_string()
        );
    }

    #[test]
    fn should_emit_passthrough_with_format_string_and_args() {
        let output = expand(quote!("entity={}", entity));
        assert_eq!(
            output.to_string(),
            quote! { debug_assert!(false, "entity={}", entity) }.to_string()
        );
    }

    #[test]
    fn should_emit_passthrough_with_captured_format_arg() {
        let output = expand(quote!("entity={entity}"));
        assert_eq!(
            output.to_string(),
            quote! { debug_assert!(false, "entity={entity}") }.to_string()
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "reached unreachable code")]
    fn should_panic_in_debug_with_no_args() {
        debug_assert!(false, "reached unreachable code");
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "specific message")]
    fn should_panic_in_debug_with_message() {
        debug_assert!(false, "specific message");
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn should_be_noop_in_release() {
        debug_assert!(false, "silently ignored");
    }
}
