use crate::MUTATIS_ATTRIBUTE_NAME;
use syn::{punctuated::Punctuated, *};

pub struct ContainerAttributes {
    /// An override of the derived mutator's name.
    ///
    /// ```ignore
    /// #[mutatis(mutator_name = FancyMutatorName)]
    /// ```
    pub mutator_name: Option<Ident>,

    /// An optional documentation string for the derived mutator.
    ///
    /// ```ignore
    /// #[mutatis(mutator_doc = "A fancy doc comment for my derived mutator!")]
    /// #[mutatis(mutator_doc = "and it can have multiple lines!")]
    /// ```
    pub mutator_doc: Option<Vec<LitStr>>,

    /// An optional flag to specify whether the derived mutator should implement
    /// `DefaultMutate` for the type or not. The default behavior is `true`.
    ///
    /// ```ignore
    /// #[mutatis(default_mutate = false)]
    /// ```
    pub default_mutate: Option<bool>,
}

impl ContainerAttributes {
    pub fn from_derive_input(derive_input: &DeriveInput) -> Result<Self> {
        let mut mutator_name = None;
        let mut mutator_doc = None;
        let mut default_mutate = None;

        for attr in &derive_input.attrs {
            if !attr.path().is_ident(MUTATIS_ATTRIBUTE_NAME) {
                continue;
            }

            let meta_list = match attr.meta {
                Meta::List(ref l) => l,
                _ => {
                    return Err(Error::new_spanned(
                        attr,
                        format!(
                            "invalid `{}` attribute. expected list",
                            MUTATIS_ATTRIBUTE_NAME
                        ),
                    ))
                }
            };

            for nested_meta in
                meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
            {
                match nested_meta {
                    Meta::NameValue(MetaNameValue {
                        path,
                        value:
                            Expr::Path(ExprPath {
                                attrs,
                                qself: None,
                                path: name,
                            }),
                        ..
                    }) if path.is_ident("mutator_name")
                        && attrs.is_empty()
                        && name.get_ident().is_some() =>
                    {
                        if mutator_name.is_some() {
                            return Err(Error::new_spanned(
                                attr,
                                format!(
                                    "invalid `{MUTATIS_ATTRIBUTE_NAME}` attribute: duplicate `mutator_name`",
                                ),
                            ));
                        }
                        mutator_name = Some(name.get_ident().unwrap().clone());
                    }

                    Meta::NameValue(MetaNameValue {
                        path,
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Bool(bool_lit),
                                ..
                            }),
                        ..
                    }) if path.is_ident("default_mutate") => {
                        if default_mutate.is_some() {
                            return Err(Error::new_spanned(
                                attr,
                                format!(
                                    "invalid `{MUTATIS_ATTRIBUTE_NAME}` attribute: duplicate `default_mutate`",
                                ),
                            ));
                        }
                        default_mutate = Some(bool_lit.value);
                    }

                    Meta::NameValue(MetaNameValue {
                        path,
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }),
                        ..
                    }) if path.is_ident("mutator_doc") => {
                        mutator_doc.get_or_insert_with(Vec::new).push(lit_str);
                    }

                    _ => {
                        return Err(Error::new_spanned(
                            attr,
                            format!("invalid `{MUTATIS_ATTRIBUTE_NAME}` attribute"),
                        ))
                    }
                }
            }
        }

        Ok(Self {
            mutator_name,
            mutator_doc,
            default_mutate,
        })
    }
}
