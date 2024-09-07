// References:
// https://github.com/dtolnay/proc-macro-workshop/tree/master
// https://www.youtube.com/watch?v=geovSK3wMB8
// https://veykril.github.io/tlborm/introduction.html
// https://users.rust-lang.org/t/checking-whether-four-2-bit-uints-are-unique-how-to-optimize/113834/12
// https://veykril.github.io/tlborm/proc-macros/methodical/attr.html
use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned as _, ItemFn};

/// Generate a compile time lookup table given a const function
///
/// The const function is currently only allowed to accept a single parameter of type
/// u8 and return type of bool, u8, u16, or u32
///
/// # Example:
/// ```
/// #[constable::lookup]
/// const fn foo(packed: u8) -> bool {
///    // divide an 8-bit integer into 4 2-bit values
///    // return true if the xor of the first 2 is equal
///    // to the xor of the second 2
///    let u0 = packed & 0b11;
///    let u1 = (packed >> 2) & 0b11;
///    let u2 = (packed >> 4) & 0b11;
///    let u3 = (packed >> 6) & 0b11;
///    (u0 ^ u1) == (u2 ^ u3)
/// }
/// fn main() {
///     let x = foo(5);
/// }
/// ```
///
///
#[proc_macro_attribute]
pub fn lookup(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // checks with syn
    // 1. must be a const function
    if input.sig.constness.is_none() {
        return syn::Error::new(
            input.sig.constness.span(),
            "constable: function must be const",
        )
        .to_compile_error()
        .into();
    }

    // 2. restrict return type
    let return_type = match &input.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => {
            if let syn::Type::Path(type_path) = ty as &syn::Type {
                let ident = type_path.path.segments.last().unwrap().ident.to_string();
                match ident.as_str() {
                    "bool" | "u8" | "u16" | "u32" => Some(ident.as_str().to_owned()),
                    _ => None,
                }
            } else {
                None
            }
        }
    };
    let Some(return_type) = return_type else {
        panic!("return type isn't one of bool, u8, u16, u32");
    };

    // 3. restrict to single input param of type u8
    let Some(input_type) = (|| {
        let params = &input.sig.inputs;

        if params.len() != 1 {
            return None;
        }

        if let syn::FnArg::Typed(syn::PatType { ty, .. }) = &params[0] {
            if let syn::Type::Path(type_path) = &**ty {
                if let Some(segment) = type_path.path.segments.first() {
                    if segment.ident == "u8" {
                        return Some("u8");
                    }
                }
            }
        }

        None
    })() else {
        panic!("input param isn't a single u8");
    };

    // rewrite the function by wrapping with a table
    let mut inner_const_fn = input.clone();
    let name = input.sig.ident.clone();

    let inner_const_fn_name = syn::Ident::new(&format!("{name}_orig"), name.span());
    inner_const_fn.sig.ident = inner_const_fn_name.clone();

    match return_type.as_str() {
        "bool" => {
            // this version does bitpacking
            quote::quote! {
                #[inline]
                pub const fn #name(value: u8) -> bool {
                    #inner_const_fn

                    type T = u8;
                    type S = u8; // it seems the assembly output emitted is slightly better when this is u8

                    const N: usize = 1 << (8 * std::mem::size_of::<T>());
                    const M: usize = N / (8 * std::mem::size_of::<S>());
                    const SHIFT: u8 = 8 * std::mem::size_of::<T>() as u8 - M.ilog2() as u8;
                    const MASK: u8 = (1 << SHIFT) - 1;

                    const TABLE: [S; M] = const {
                        let mut table: [S; M] = [0; M];

                        let mut i = 0;
                        while i < N {
                            let outer = (i as u8) >> SHIFT;
                            let inner = (i as u8) & MASK;

                            table[outer as usize] |= (#inner_const_fn_name(i as u8) as S) << inner;
                            i += 1;
                        }

                        table
                    };

                    (TABLE[(value >> SHIFT) as usize] >> (value & MASK)) & 1 == 1
                }
            }
            .into()
        }
        "u8" | "u16" | "u32" => {
            let in_type = input_type.to_string();
            let out_type = return_type.to_string();
            quote::quote! {
            #[inline]
            pub const fn #name(value: u8) -> bool {
                #inner_const_fn

                type T = #in_type;
                type S = #out_type;
                const N: usize = 1 << (8 * std::mem::size_of::<T>());

                const TABLE: [S; N] = const {
                    let mut table: [S; N] = [0; N];

                    let mut i = 0;
                    while i < N {
                        table[i as usize] = #inner_const_fn_name(i as u8);
                        i += 1;
                    }

                    table
                };

                TABLE[value as usize]
            }

            }
            .into()
        }
        _ => unreachable!(),
    }
}
