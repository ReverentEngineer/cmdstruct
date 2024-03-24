use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    parse::Parser,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute,
    AttrStyle,
    Expr,
    ExprLit,
    Lit,
    Field,
    FieldsNamed,
    Ident,
    ItemStruct,
    LitStr,
    MetaNameValue,
    Type
};

type Result<T> = std::result::Result<T, syn::Error>;

type Attributes = Punctuated<syn::Meta, syn::Token![,]>;

#[proc_macro_attribute]
pub fn command(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    match Command::parse(attrs, item_struct) {
        Ok(command) => command.into(),
        Err(err) => err.into_compile_error().into()
    }
}


struct CommandAttributes {
    executable: String
}

fn get_string(expr: &syn::Expr) -> Result<String> {
    match expr {
        Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(litstr) })
            => Ok(litstr.value()),
        _ => Err(syn::Error::new(expr.span(), "Expected a string literal")),
    }
}

impl CommandAttributes {

    fn parse(attrs: Attributes) -> Result<Self> {
        let mut executable = None;
        for attr in &attrs {
            match &attr {
                syn::Meta::NameValue(MetaNameValue { 
                    path,
                    eq_token: _,
                    value
                }) => {
                    if path.is_ident("executable") {
                        executable = match get_string(value) {
                            Ok(s) => Some(s),
                            _ => return Err(syn::Error::new(value.span(),
                            "Unexpected value of 'executable'"))
                        };
                    } else{
                        return Err(syn::Error::new(attr.span(), "Unsupported attribute"))
                    }
                },
                _ => return Err(syn::Error::new(attr.span(), "Unsupported attribute type")),
            }
        }
        if let Some(executable) = executable {
            Ok(Self {
                executable 
            })
        } else {
            Err(syn::Error::new(attrs.span(), "No 'executable' defined for 'command'"))
        }
    }
}

struct Command {
    attributes: CommandAttributes,
    ident: Ident,
    args: Vec<Arg>,
    item_struct: ItemStruct
}

impl Command {

    fn parse(attrs: TokenStream, mut item_struct: ItemStruct) -> Result<Command> {
        let attributes = Attributes::parse_terminated
            .parse2(attrs.into())
            .and_then(CommandAttributes::parse)?;
        let args: Vec<Arg> = match &mut item_struct.fields {
            syn::Fields::Named(FieldsNamed { 
                brace_token: _,
                named 
            }) => named.iter_mut().filter_map(collect_arg).collect(),
            _ => Err(syn::Error::new(item_struct.span(),
            "command only supports struct wit named fields"))
        }?;
        Ok(Command {
            attributes,
            ident: item_struct.ident.clone(),
            args,
            item_struct
        })
    }

}



enum ArgType {
    Option {
        name: String
    },
    Flag {
        name: String
    },
    Positional
}

#[allow(dead_code)]
struct Arg {
    arg_type: ArgType,
    ident: Ident,
    ty: Type
}

type ArgResult = Result<(Option<Attribute>, Option<ArgType>)>;

fn parse_arg_with_attributes(attr: Attribute) -> ArgResult {
    let mut arg_type = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("option") {
            if arg_type.is_none() {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                arg_type = Some(ArgType::Option {
                    name: s.value()
                });
                Ok(())
            } else {
                Err(meta.error("Only one argument type allowed."))
            }
        } else if meta.path.is_ident("flag") {
            if arg_type.is_none() {
                let value = meta.value()?;
                let s: LitStr = value.parse()?;
                arg_type = Some(ArgType::Flag {
                    name: s.value()
                });
                Ok(())
            } else {
                Err(meta.error("Only one argument type allowed."))
            }
        } else {
            Err(meta.error("Unrecognized arg"))
        }
    }).map(|_| {
        arg_type.map_or((Some(attr), None), |arg_type| (None, Some(arg_type)))
    })
}

fn map_to_attr_or_arg(attr: Attribute) -> ArgResult {
    match attr.style {
        AttrStyle::Outer => match &attr.meta {
            syn::Meta::List(list) if list.path.is_ident("arg")
                => parse_arg_with_attributes(attr),
                syn::Meta::Path(path) if path.is_ident("arg") =>
                    Ok((None, Some(ArgType::Positional))),
                _  => Ok((Some(attr), None))
        },
        _  => Ok((Some(attr), None))
    }
}

fn collect_arg(field: &mut Field) -> Option<Result<Arg>> {
    if let Some(ident) = &field.ident {
        let arg_results: Result<Vec<_>> = field.attrs.clone()
            .into_iter().map(map_to_attr_or_arg).collect();
        match arg_results {
            Ok(results) => {
                let unzipped: (Vec<_>, Vec<_>) = results.into_iter().unzip();
                match unzipped {
                    (attrs, arg_types) => {
                        let attrs: Vec<_> = attrs.into_iter()
                            .filter_map(|attr| attr).collect();
                        let mut arg_types: Vec<_> = arg_types.into_iter()
                            .filter_map(|arg_type| arg_type).collect();
                        field.attrs = attrs;
                        match arg_types.len() {
                            1 => Some(Ok(Arg {
                                arg_type: arg_types.remove(0),
                                ident: ident.clone(),
                                ty: field.ty.clone()
                            })),
                            0 => None,
                            _ => Some(Err(syn::Error::new(field.span(), "Too many args")))
                        }
                    },
                }
            },
            Err(err) => Some(Err(err))
        }
    } else {
        None
    }
}

fn append_arg_tokens(arg: &Arg) -> proc_macro2::TokenStream {
    let ident = &arg.ident;
    match &arg.arg_type {
        ArgType::Option { name } => quote! { 
            args.push(#name);
            args.push(&self.#ident);
        },
        ArgType::Flag { name } => quote! {  
            if self.#ident {
                args.push(#name);
            }
        },
        ArgType::Positional => quote! {
            args.push(&self.#ident);
        }
    }
}


impl Into<TokenStream> for Command {

    fn into(self) -> TokenStream {
        let args: Vec<_> = self.args.iter().map(append_arg_tokens).collect();
        let executable = &self.attributes.executable;
        let struct_ident = &self.ident;
        let item_struct = &self.item_struct;
        let impls_combined = quote! {

            #item_struct

            impl #struct_ident {

                pub fn command(&self) -> std::process::Command {
                    let mut command = std::process::Command::new(#executable);
                    let mut args: Vec<&str> = Vec::new();
                    #(#args)*
                    command.args(args);
                    command
                }
            }
        };
        impls_combined.into()
    }

}
