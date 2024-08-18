extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, *};

mod container_attributes;
mod field_attributes;
use container_attributes::ContainerAttributes;
use field_attributes::FieldBehavior;

static MUTATIS_ATTRIBUTE_NAME: &str = "mutatis";

#[proc_macro_derive(Mutate, attributes(mutatis))]
pub fn derive_mutator(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as DeriveInput);
    expand_derive_mutator(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_derive_mutator(input: DeriveInput) -> Result<TokenStream> {
    let container_attrs = ContainerAttributes::from_derive_input(&input)?;
    let mutator_ty = MutatorType::new(&input, &container_attrs)?;

    let mutator_type_def = gen_mutator_type_def(&input, &mutator_ty, &container_attrs)?;
    let mutator_type_default_impl = gen_mutator_type_default_impl(&mutator_ty)?;
    let mutator_ctor = gen_mutator_ctor(&mutator_ty)?;
    let mutator_impl = gen_mutator_impl(&input, &mutator_ty)?;
    let default_mutator_impl = gen_default_mutator_impl(&mutator_ty, &container_attrs)?;

    Ok(quote! {
        #mutator_type_def
        #mutator_type_default_impl
        #mutator_ctor
        #mutator_impl
        #default_mutator_impl
    })
}

struct MutatorType {
    ty_name: Ident,

    mutator_name: Ident,

    mutator_fields: Vec<MutatorField>,

    /// A vec of quoted generic parameters, without any bounds but with `const`
    /// defs, e.g. `'a`, `const N: usize`, or `T`.
    ty_impl_generics: Vec<TokenStream>,

    /// A vec of quoted generic parameters, without any bounds and without any
    /// `const` defs, e.t. `'a`, `N`, or `T.
    ty_name_generics: Vec<TokenStream>,

    /// A vec of quoted bounds for the generics above, e.g. `A: Iterator<Item =
    /// B>,`.
    ty_generics_bounds: Vec<TokenStream>,
}

impl MutatorType {
    fn new(input: &DeriveInput, container_attrs: &ContainerAttributes) -> Result<Self> {
        let ty_name = input.ident.clone();

        let mutator_name = container_attrs
            .mutator_name
            .clone()
            .unwrap_or_else(|| Ident::new(&format!("{}Mutator", input.ident), input.ident.span()));

        let mutator_fields = get_mutator_fields(&input)?;

        let mut ty_impl_generics = vec![];
        let mut ty_name_generics = vec![];
        let mut ty_generics_bounds = vec![];

        for gen in &input.generics.params {
            match gen {
                GenericParam::Lifetime(l) => {
                    if !l.bounds.is_empty() {
                        ty_generics_bounds.push(quote! { #l });
                    }

                    let l = &l.lifetime;
                    ty_impl_generics.push(quote! { #l });
                    ty_name_generics.push(quote! { #l });
                }
                GenericParam::Const(c) => {
                    ty_impl_generics.push(quote! { #c });
                    let c = &c.ident;
                    ty_name_generics.push(quote! { #c });
                }
                GenericParam::Type(t) => {
                    if !t.bounds.is_empty() {
                        ty_generics_bounds.push(quote! { #t });
                    }
                    let t = &t.ident;
                    ty_impl_generics.push(quote! { #t });
                    ty_name_generics.push(quote! { #t });
                }
            }
        }

        if let Some(wc) = &input.generics.where_clause {
            for bound in wc.predicates.iter() {
                ty_generics_bounds.push(quote! { #bound });
            }
        }

        Ok(Self {
            ty_name,
            mutator_name,
            mutator_fields,
            ty_impl_generics,
            ty_name_generics,
            ty_generics_bounds,
        })
    }

    fn mutator_impl_generics_iter(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.ty_impl_generics.iter().cloned().chain(
            self.mutator_fields
                .iter()
                .filter_map(|f| f.generic.as_ref().map(|g| quote! { #g })),
        )
    }

    /// All the `impl` generic parameters for this mutator, including those
    /// inherited from the type that it is a mutator for.
    fn mutator_impl_generics(&self) -> TokenStream {
        let impl_generics = self
            .ty_impl_generics
            .iter()
            .cloned()
            .chain(
                self.mutator_fields
                    .iter()
                    .filter_map(|f| f.generic.as_ref().map(|g| quote! { #g })),
            )
            .collect::<Vec<_>>();
        if impl_generics.is_empty() {
            quote! {}
        } else {
            quote! { < #( #impl_generics ),* > }
        }
    }

    /// All the named (i.e. just the "N" and excluding "const", ":", and "usize"
    /// in `const N: usize` generics) generic parameters for this mutator,
    /// including those inherited from the type that it is a mutator for.
    fn mutator_name_generics_iter(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.ty_name_generics.iter().cloned().chain(
            self.mutator_fields
                .iter()
                .filter_map(|f| f.generic.as_ref().map(|g| quote! { #g })),
        )
    }

    fn mutator_impl_generics_with_defaults_iter(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.ty_impl_generics
            .iter()
            .cloned()
            .chain(self.mutator_fields.iter().filter_map(move |f| {
                f.generic.as_ref().map(|g| {
                    let for_ty = &f.for_ty;
                    quote! { #g = <#for_ty as mutatis::DefaultMutate>::DefaultMutate }
                })
            }))
    }

    fn ty_name_with_generics(&self) -> TokenStream {
        let ty_name = &self.ty_name;
        if self.ty_name_generics.is_empty() {
            quote! { #ty_name }
        } else {
            let ty_generics = self.ty_name_generics.iter();
            quote! { #ty_name < #( #ty_generics ),* > }
        }
    }

    fn mutator_name_with_generics(&self, kind: MutatorNameGenericsKind) -> TokenStream {
        let mutator_name = &self.mutator_name;

        let generics = match kind {
            MutatorNameGenericsKind::Generics => {
                self.mutator_name_generics_iter().collect::<Vec<_>>()
            }
            MutatorNameGenericsKind::Impl {
                impl_default: false,
            } => self.mutator_impl_generics_iter().collect::<Vec<_>>(),
            MutatorNameGenericsKind::Impl { impl_default: true } => self
                .mutator_impl_generics_with_defaults_iter()
                .collect::<Vec<_>>(),
            MutatorNameGenericsKind::JustTyGenerics => self.ty_name_generics.clone(),
        };

        if generics.is_empty() {
            quote! { #mutator_name }
        } else {
            quote! { #mutator_name < #( #generics ),* > }
        }
    }

    fn where_clause(&self, kind: WhereClauseKind) -> TokenStream {
        let mut bounds = self.ty_generics_bounds.clone();

        match kind {
            WhereClauseKind::NoMutateBounds => {}
            WhereClauseKind::MutateBounds => {
                for f in &self.mutator_fields {
                    let for_ty = &f.for_ty;
                    if let Some(g) = f.generic.as_ref() {
                        bounds.push(quote! { #g: mutatis::Mutate<#for_ty> });
                    } else {
                        debug_assert_eq!(f.behavior, FieldBehavior::DefaultMutate);
                        bounds.push(quote! { #for_ty: mutatis::DefaultMutate });
                    }
                }
            }
            WhereClauseKind::DefaultBounds => {
                for f in &self.mutator_fields {
                    if let Some(g) = f.generic.as_ref() {
                        bounds.push(quote! { #g: Default });
                    } else {
                        let for_ty = &f.for_ty;
                        debug_assert_eq!(f.behavior, FieldBehavior::DefaultMutate);
                        bounds.push(quote! { #for_ty: mutatis::DefaultMutate });
                    }
                }
            }
            WhereClauseKind::DefaultMutateBounds => {
                for f in &self.mutator_fields {
                    let for_ty = &f.for_ty;
                    bounds.push(quote! { #for_ty: mutatis::DefaultMutate });
                }
            }
        }

        if bounds.is_empty() {
            quote! {}
        } else {
            quote! { where #( #bounds ),* }
        }
    }

    fn phantom_fields_defs<'a>(
        &self,
        input: &'a DeriveInput,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let make_phantom_field = |i, ty| {
            let ident = Ident::new(&format!("_phantom{i}"), Span::call_site());
            quote! { #ident : core::marker::PhantomData<#ty> , }
        };

        input
            .generics
            .params
            .iter()
            .enumerate()
            .map(move |(i, g)| match g {
                GenericParam::Lifetime(l) => {
                    let l = &l.lifetime;
                    make_phantom_field(i, quote! { & #l () })
                }
                GenericParam::Const(c) => {
                    let c = &c.ident;
                    make_phantom_field(i, quote! { [(); #c] })
                }
                GenericParam::Type(t) => {
                    let t = &t.ident;
                    make_phantom_field(i, quote! { #t })
                }
            })
    }

    fn phantom_fields_literals(&self) -> impl Iterator<Item = TokenStream> + '_ {
        (0..self.ty_name_generics.len()).map(|i| {
            let ident = Ident::new(&format!("_phantom{i}"), Span::call_site());
            quote! { #ident : core::marker::PhantomData, }
        })
    }
}

#[derive(Clone, Copy)]
enum WhereClauseKind {
    NoMutateBounds,
    MutateBounds,
    DefaultBounds,
    DefaultMutateBounds,
}

#[derive(Clone, Copy)]
enum MutatorNameGenericsKind {
    Generics,
    Impl { impl_default: bool },
    JustTyGenerics,
}

struct MutatorField {
    /// The identifier for this field inside the mutator struct.
    ident: Ident,
    /// The generic type parameter for this field, if any.
    generic: Option<Ident>,
    /// The behavior for this field.
    behavior: FieldBehavior,
    /// The type that this field is a mutator for.
    for_ty: Type,
}

fn get_mutator_fields(input: &DeriveInput) -> Result<Vec<MutatorField>> {
    let mut i = 0;
    let mut generic = |b: &FieldBehavior| -> Option<Ident> {
        if b.needs_generic() {
            let g = Ident::new(&format!("MutatorT{}", i), Span::call_site());
            i += 1;
            Some(g)
        } else {
            None
        }
    };

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|f| {
                    FieldBehavior::for_field(f)
                        .map(|b| {
                            b.map(|b| MutatorField {
                                ident: f.ident.clone().unwrap(),
                                generic: generic(&b),
                                behavior: b,
                                for_ty: f.ty.clone(),
                            })
                        })
                        .transpose()
                })
                .collect(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .filter_map(|(i, f)| {
                    FieldBehavior::for_field(f)
                        .map(|b| {
                            b.map(|b| MutatorField {
                                ident: Ident::new(&format!("field{}", i), f.span()),
                                generic: generic(&b),
                                behavior: b,
                                for_ty: f.ty.clone(),
                            })
                        })
                        .transpose()
                })
                .collect(),
            Fields::Unit => Ok(vec![]),
        },
        Data::Enum(data) => Ok(data
            .variants
            .iter()
            .map(|v| {
                let prefix = v.ident.to_string().to_lowercase();
                match v.fields {
                    Fields::Named(ref fields) => fields
                        .named
                        .iter()
                        .filter_map(|f| {
                            FieldBehavior::for_field(f)
                                .map(|b| {
                                    b.map(|b| MutatorField {
                                        ident: Ident::new(
                                            &format!("{prefix}_{}", f.ident.clone().unwrap()),
                                            f.span(),
                                        ),
                                        generic: generic(&b),
                                        behavior: b,
                                        for_ty: f.ty.clone(),
                                    })
                                })
                                .transpose()
                        })
                        .collect::<Result<Vec<_>>>(),
                    Fields::Unnamed(ref fields) => fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .filter_map(|(i, f)| {
                            FieldBehavior::for_field(f)
                                .map(|b| {
                                    b.map(|b| MutatorField {
                                        ident: Ident::new(&format!("{prefix}{i}"), f.span()),
                                        generic: generic(&b),
                                        behavior: b,
                                        for_ty: f.ty.clone(),
                                    })
                                })
                                .transpose()
                        })
                        .collect::<Result<Vec<_>>>(),
                    Fields::Unit => Ok(vec![]),
                }
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flat_map(|fs| fs)
            .collect()),
        Data::Union(_) => Err(Error::new_spanned(
            input,
            "cannot `derive(Mutate)` on a union",
        )),
    }
}

fn gen_mutator_type_def(
    input: &DeriveInput,
    mutator_ty: &MutatorType,
    container_attrs: &ContainerAttributes,
) -> Result<TokenStream> {
    let vis = &input.vis;
    let name = &input.ident;

    let impl_default = container_attrs.default_mutator.unwrap_or(true);
    let mutator_name =
        mutator_ty.mutator_name_with_generics(MutatorNameGenericsKind::Impl { impl_default });

    let mut temp: Option<LitStr> = None;
    let doc = container_attrs.mutator_doc.as_deref().unwrap_or_else(|| {
        temp = Some(LitStr::new(
            &format!(" A mutator for the `{name}` type."),
            input.ident.span(),
        ));
        std::slice::from_ref(temp.as_ref().unwrap())
    });

    let where_clause = mutator_ty.where_clause(WhereClauseKind::NoMutateBounds);

    let fields = mutator_ty
        .mutator_fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if let Some(g) = f.generic.as_ref() {
                quote! { #ident: #g , }
            } else {
                let for_ty = &f.for_ty;
                debug_assert_eq!(f.behavior, FieldBehavior::DefaultMutate);
                quote! { #ident: <#for_ty as mutatis::DefaultMutate>::DefaultMutate, }
            }
        })
        .collect::<Vec<_>>();

    let phantoms = mutator_ty.phantom_fields_defs(input);

    Ok(quote! {
        #( #[doc = #doc] )*
        // #[derive(Clone, Debug)]
        #vis struct #mutator_name #where_clause {
            #( #fields )*
            #( #phantoms )*
            _private: (),
        }
    })
}

fn gen_mutator_type_default_impl(mutator_ty: &MutatorType) -> Result<TokenStream> {
    let impl_generics = mutator_ty.mutator_impl_generics();
    let mutator_name = mutator_ty.mutator_name_with_generics(MutatorNameGenericsKind::Generics);
    let where_clause = mutator_ty.where_clause(WhereClauseKind::DefaultBounds);

    let fields = mutator_ty
        .mutator_fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            quote! { #ident: Default::default(), }
        })
        .collect::<Vec<_>>();

    let phantoms = mutator_ty.phantom_fields_literals();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics Default for #mutator_name #where_clause {
            fn default() -> Self {
                Self {
                    #( #fields )*
                    #( #phantoms )*
                    _private: (),
                }
            }
        }
    })
}

fn gen_mutator_ctor(mutator_ty: &MutatorType) -> Result<TokenStream> {
    let impl_generics = mutator_ty.mutator_impl_generics();

    let params = mutator_ty
        .mutator_fields
        .iter()
        .filter_map(|f| {
            f.generic.as_ref().map(|g| {
                let ident = &f.ident;
                quote! { #ident: #g , }
            })
        })
        .collect::<Vec<_>>();

    let fields = mutator_ty
        .mutator_fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if f.generic.is_some() {
                quote! { #ident , }
            } else {
                let for_ty = &f.for_ty;
                debug_assert_eq!(f.behavior, FieldBehavior::DefaultMutate);
                quote! { #ident: mutatis::mutators::default::<#for_ty>() , }
            }
        })
        .collect::<Vec<_>>();

    let name = &mutator_ty.mutator_name_with_generics(MutatorNameGenericsKind::Generics);
    let doc = format!("Construct a new `{name}` instance.");
    let where_clause = mutator_ty.where_clause(WhereClauseKind::NoMutateBounds);
    let phantoms = mutator_ty.phantom_fields_literals();

    Ok(quote! {
        impl #impl_generics #name #where_clause {
            #[doc = #doc]
            #[inline]
            pub fn new( #( #params )* ) -> Self {
                Self {
                    #( #fields )*
                    #( #phantoms )*
                    _private: (),
                }
            }
        }
    })
}

fn gen_mutator_impl(input: &DeriveInput, mutator_ty: &MutatorType) -> Result<TokenStream> {
    // TODO: make a list of all the individual mutations we *could* make, and
    // then choose only one of them to actually perform.

    let impl_generics = mutator_ty.mutator_impl_generics();

    let ty_name = mutator_ty.ty_name_with_generics();
    let where_clause = mutator_ty.where_clause(WhereClauseKind::MutateBounds);

    let mut fields_iter = mutator_ty.mutator_fields.iter();
    let mut make_mutation = |value| {
        let ident = &fields_iter.next().unwrap().ident;
        quote! { self.#ident.mutate(mutations, #value)?; }
    };

    let mutation_body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let mutations = fields
                    .named
                    .iter()
                    .filter(|f| FieldBehavior::for_field(f).unwrap().is_some())
                    .map(|f| {
                        let ident = &f.ident;
                        make_mutation(quote! { &mut value.#ident })
                    });
                quote! {
                    #( #mutations )*
                }
            }
            Fields::Unnamed(fields) => {
                let mutations = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter(|(_i, f)| FieldBehavior::for_field(f).unwrap().is_some())
                    .map(|(i, f)| {
                        let index = Index {
                            index: u32::try_from(i).unwrap(),
                            span: f.span(),
                        };
                        make_mutation(quote! { &mut value.#index })
                    });
                quote! {
                    #( #mutations )*
                }
            }
            Fields::Unit => quote! {},
        },

        Data::Enum(data) => {
            // TODO: add support for changing from one enum variant to another.

            let mut variants = vec![];
            for v in data.variants.iter() {
                let variant_ident = &v.ident;
                match &v.fields {
                    Fields::Named(fields) => {
                        let mut patterns = vec![];
                        let mutates = fields
                            .named
                            .iter()
                            .filter_map(|f| {
                                let ident = &f.ident;
                                if FieldBehavior::for_field(f).unwrap().is_some() {
                                    patterns.push(quote! { #ident , });
                                    Some(make_mutation(quote! { #ident }))
                                } else {
                                    patterns.push(quote! { #ident: _ , });
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        variants.push(quote! {
                            #ty_name::#variant_ident { #( #patterns )* } => {
                                #( #mutates )*
                            }
                        });
                    }

                    Fields::Unnamed(fields) => {
                        let mut patterns = vec![];
                        let mutates = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .filter_map(|(i, f)| {
                                if FieldBehavior::for_field(f).unwrap().is_some() {
                                    let binding = Ident::new(&format!("field{}", i), f.span());
                                    patterns.push(quote! { #binding , });
                                    Some(make_mutation(quote! { #binding }))
                                } else {
                                    patterns.push(quote! { _ , });
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        variants.push(quote! {
                            #ty_name::#variant_ident( #( #patterns )* ) => {
                                #( #mutates )*
                            }
                        });
                    }

                    Fields::Unit => {
                        variants.push(quote! {
                            #ty_name::#variant_ident => {}
                        });
                    }
                }
            }

            quote! {
                match value {
                    #( #variants )*
                }
            }
        }

        Data::Union(_) => {
            return Err(Error::new_spanned(
                input,
                "cannot `derive(Mutate)` on a union",
            ))
        }
    };

    let mutate_method = quote! {
        fn mutate(
            &mut self,
            mutations: &mut mutatis::Candidates,
            value: &mut #ty_name,
        ) -> mutatis::Result<()> {
            #mutation_body

            // Silence unused-variable warnings if every field was marked `ignore`.
            let _ = (mutations, value);

            Ok(())
        }
    };

    let mutator_name = &mutator_ty.mutator_name_with_generics(MutatorNameGenericsKind::Generics);

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics mutatis::Mutate<#ty_name> for #mutator_name
            #where_clause
        {
            #mutate_method
        }
    })
}

fn gen_default_mutator_impl(
    mutator_ty: &MutatorType,
    container_attrs: &ContainerAttributes,
) -> Result<TokenStream> {
    let impl_default = container_attrs.default_mutator.unwrap_or(true);
    if !impl_default {
        return Ok(quote! {});
    }

    let ty_generics = if mutator_ty.ty_impl_generics.is_empty() {
        quote! {}
    } else {
        let gens = &mutator_ty.ty_impl_generics;
        quote! { < #( #gens ),* > }
    };

    let ty_name = mutator_ty.ty_name_with_generics();
    let where_clause = mutator_ty.where_clause(WhereClauseKind::DefaultMutateBounds);
    let mutator_name =
        &mutator_ty.mutator_name_with_generics(MutatorNameGenericsKind::JustTyGenerics);

    Ok(quote! {
        #[automatically_derived]
        impl #ty_generics mutatis::DefaultMutate for #ty_name
            #where_clause
        {
            type DefaultMutate = #mutator_name;
        }
    })
}
