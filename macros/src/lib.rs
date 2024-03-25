use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input,
    spanned::Spanned,
    Attribute,
    AttrStyle,
    Data,
    DataStruct,
    DeriveInput,
    Field,
    Fields,
    FieldsNamed,
    Ident,
    LitStr,
    MetaList,
    Path,
    Type
};

type Result<T> = std::result::Result<T, syn::Error>;

#[proc_macro_derive(Command, attributes(command, arg))]
pub fn command(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match Command::parse(derive_input) {
        Ok(command) => command.into(),
        Err(err) => err.into_compile_error().into()
    }
}

enum Executable {
    Const(String),
    Function(Path)
}

struct CommandAttributes {
    executable: Executable,
}

impl CommandAttributes {
    fn parse(derive_input: &DeriveInput) -> Result<Self> {
        let mut executable = None;
        for attr in &derive_input.attrs {
            if attr.path().is_ident("command") {
                match &attr.meta {
                    syn::Meta::List(MetaList {
                        path: _,
                        delimiter: _,
                        tokens: _,
                    }) => {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("executable") {
                                let value = meta.value()?;
                                let s: LitStr = value.parse()?;
                                executable = Some(Executable::Const(s.value()));
                                Ok(())
                            } else if meta.path.is_ident("executable_fn") {
                                let value = meta.value()?;
                                let s: Path = value.parse()?;
                                executable = Some(Executable::Function(s));
                                Ok(())
                            } else {
                                return Err(syn::Error::new(attr.span(), "Unsupported attribute"));
                            }
                        })?;
                    }
                    _ => {}
                }
            }
        }
        if let Some(executable) = executable {
            Ok(Self { executable })
        } else {
            Err(syn::Error::new(
                derive_input.span(),
                "No 'executable' defined for 'command'",
            ))
        }
    }
}

struct Command {
    attributes: CommandAttributes,
    ident: Ident,
    args: Vec<Arg>
}

impl Command {

    fn parse(derive_input: DeriveInput) -> Result<Command> {
        let attributes = CommandAttributes::parse(&derive_input)?;

        let args = match derive_input.data {
            Data::Struct(DataStruct {
                struct_token: _,
                fields: Fields::Named(
                    FieldsNamed {
                        brace_token: _,
                        mut named
                    }
                ),
                semi_token: _
            }) => named.iter_mut().filter_map(collect_arg).collect(),
            _ => Err(syn::Error::new(derive_input.span(),
            "Only structs with named fields supported."))
        }?;
        Ok(Command {
            attributes,
            ident: derive_input.ident.clone(),
            args
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
            cmdstruct::Arg::append_option(&self.#ident, #name, &mut command);
        },
        ArgType::Flag { name } => quote! {
            if self.#ident {
                command.arg(#name);
            }
        },
        ArgType::Positional => quote! {
            cmdstruct::Arg::append_arg(&self.#ident, &mut command);
        },
    }
}

impl Into<TokenStream> for Command {
    fn into(self) -> TokenStream {
        let args: Vec<_> = self.args.iter().map(append_arg_tokens).collect();
        let executable = match &self.attributes.executable {
            Executable::Const(executable) => quote! { #executable },
            Executable::Function(func) => quote! { #func(&self) },
        };
        let struct_ident = &self.ident;
        let impls_combined = quote! {

            impl #struct_ident {

                pub fn command(&self) -> std::process::Command {
                    let mut command = std::process::Command::new(#executable);
                    #(#args)*
                    command
                }
            }
        };
        impls_combined.into()
    }

}
