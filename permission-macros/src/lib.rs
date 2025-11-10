use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Expr, Ident, ItemFn, Token};

struct PermissionArgs {
    roles: Vec<Expr>,
    permissions: Vec<Expr>,
    resource_id: Vec<Expr>,
}

impl Parse for PermissionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut roles = Vec::new();
        let mut permissions = Vec::new();
        let mut resource_id = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "roles" | "permissions" | "resource_id" => {
                    let content;
                    syn::bracketed!(content in input);
                    let list = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;

                    if ident == "roles" {
                        roles = list.into_iter().collect();
                    } else if ident == "resource_id" {
                        resource_id = list.into_iter().collect();
                    } else {
                        permissions = list.into_iter().collect();
                    }
                }
                _ => return Err(syn::Error::new_spanned(ident, "Unknown attribute")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(PermissionArgs {
            roles,
            permissions,
            resource_id,
        })
    }
}

 #[proc_macro_attribute]
pub fn permission_required(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as PermissionArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

     let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    let roles = args.roles;
    let permissions = args.permissions;
    let resource_id = args.resource_id;

    let expanded = quote! {
        #fn_vis #fn_sig {
            // 权限检查
            let required_roles = vec![#(#roles),*];
            let required_permissions = vec![#(#permissions),*];
            let resource_ids = vec![#(#resource_id),*];

            // 检查用户是否具有所需的角色和权限
            if !required_roles.is_empty() || !required_permissions.is_empty() {
                if !check_user_permissions(&req, &required_roles, &required_permissions, &resource_ids) {
                    return actix_web::HttpResponse::Forbidden()
                        .body("Insufficient permissions");
                }
            }

            #fn_block
        }
    };

    TokenStream::from(expanded)
}

