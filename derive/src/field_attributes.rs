use crate::MUTATIS_ATTRIBUTE_NAME;
use quote::quote;
use syn::{spanned::Spanned, *};

/// Determines how a value for a field should be constructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldBehavior {
    /// Mutate this field using a generic mutator (the default behavior).
    GenericMutator,

    /// Use the default mutator to mutate this field; don't add a generic
    /// mutator type parameter for it.
    DefaultMutate,
}

impl FieldBehavior {
    pub fn for_field(field: &Field) -> Result<Option<FieldBehavior>> {
        let opt_attr = fetch_attr_from_field(field)?;
        let ctor = match opt_attr {
            Some(attr) => parse_attribute(attr)?,
            None => Some(FieldBehavior::GenericMutator),
        };
        Ok(ctor)
    }

    pub fn needs_generic(&self) -> bool {
        match self {
            FieldBehavior::GenericMutator => true,
            FieldBehavior::DefaultMutate => false,
        }
    }
}

fn fetch_attr_from_field(field: &Field) -> Result<Option<&Attribute>> {
    let found_attributes: Vec<_> = field
        .attrs
        .iter()
        .filter(|a| {
            let path = a.path();
            let name = quote!(#path).to_string();
            name == MUTATIS_ATTRIBUTE_NAME
        })
        .collect();
    if found_attributes.len() > 1 {
        let name = field.ident.as_ref().unwrap();
        let msg = format!(
            "Multiple conflicting #[{MUTATIS_ATTRIBUTE_NAME}] attributes found on field `{name}`"
        );
        return Err(syn::Error::new(field.span(), msg));
    }
    Ok(found_attributes.into_iter().next())
}

fn parse_attribute(attr: &Attribute) -> Result<Option<FieldBehavior>> {
    if let Meta::List(ref meta_list) = attr.meta {
        parse_attribute_internals(meta_list)
    } else {
        let msg = format!("#[{MUTATIS_ATTRIBUTE_NAME}] must contain a group");
        Err(syn::Error::new(attr.span(), msg))
    }
}

fn parse_attribute_internals(meta_list: &MetaList) -> Result<Option<FieldBehavior>> {
    let mut tokens_iter = meta_list.tokens.clone().into_iter();
    let token = tokens_iter.next().ok_or_else(|| {
        let msg = format!("#[{MUTATIS_ATTRIBUTE_NAME}] cannot be empty.");
        syn::Error::new(meta_list.span(), msg)
    })?;
    match token.to_string().as_ref() {
        "ignore" => Ok(None),
        "default_mutator" => Ok(Some(FieldBehavior::DefaultMutate)),
        _ => {
            let msg = format!("Unknown option for #[{MUTATIS_ATTRIBUTE_NAME}]: `{token}`");
            Err(syn::Error::new(token.span(), msg))
        }
    }
}
