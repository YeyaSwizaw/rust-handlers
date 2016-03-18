use syntax::ast::*;
use syntax::ptr::P;
use syntax::codemap::{respan, Span};
use syntax::ext::base::{MacResult, MacEager};
use syntax::util::small_vector::SmallVector;
use syntax::parse::token::str_to_ident;
use syntax::abi::Abi;

use ::util;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub name: Ident,
    pub span: Span,
    pub handlers: Vec<HandlerInfo>
}

#[derive(Debug, Clone)]
pub struct HandlerInfo {
    pub name: Ident,
    pub fns: Vec<HandlerFnInfo>
}

#[derive(Debug, Clone)]
pub struct HandlerFnInfo {
    pub source_name: Ident,
    pub dest_name: Ident,
    pub args: Vec<HandlerFnArg>
}

#[derive(Debug, Clone)]
pub struct HandlerFnArg {
    pub name: Ident,
    pub ty: Ident,
}

impl SystemInfo {
    pub fn new(name: Ident, span: Span) -> SystemInfo {
        SystemInfo {
            name: name,
            span: span,
            handlers: Vec::new()
        }
    }

    pub fn add_handler(&mut self, handler: HandlerInfo) {
        self.handlers.push(handler);
    }

    fn object_name(&self) -> Ident {
        util::indent_append(self.name, str_to_ident("Object"))
    }

    fn generate_object_trait(&self) -> Item {
        let mut fns = Vec::new();

        for handler in self.handlers.iter() {
            fns.push(handler.generate_as_self(self));
            fns.push(handler.generate_as_self_mut(self));
        }

        Item {
            ident: self.object_name(),
            attrs: Vec::new(),
            node: ItemKind::Trait(
                Unsafety::Normal,
                Default::default(),
                P::from_vec(Vec::new()),
                fns
            ),
            id: DUMMY_NODE_ID,
            span: self.span,
            vis: Visibility::Public
        }
    }

    fn generate_struct(&self) -> Item {
        let objects_field = StructField_ {
            kind: StructFieldKind::NamedField(str_to_ident("objects"), Visibility::Inherited),
            id: DUMMY_NODE_ID,
            ty: P(util::param_ty_from_ident(
                str_to_ident("Vec"),
                util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                )
            )),
            attrs: Vec::new()
        };

        let mut fields = vec![respan(self.span, objects_field)];

        for handler in self.handlers.iter() {
            fields.push(respan(self.span, StructField_ {
                kind: StructFieldKind::NamedField(util::indent_append(handler.name, str_to_ident("_idxs")), Visibility::Inherited),
                id: DUMMY_NODE_ID,
                ty: P(util::param_ty_from_ident(
                    str_to_ident("Vec"),
                    util::ty_from_ident(str_to_ident("usize"))
                )),
                attrs: Vec::new()
            }));
        }

        Item {
            ident: self.name,
            attrs: Vec::new(),
            node: ItemKind::Struct(
                VariantData::Struct(
                    fields,
                    DUMMY_NODE_ID
                ),
                Default::default()
            ),
            id: DUMMY_NODE_ID,
            span: self.span,
            vis: Visibility::Public
        }
    }

    pub fn generate_ast(&self) -> Box<MacResult> {
        let object_trait = self.generate_object_trait();
        let system_struct = self.generate_struct();

        let mut items: Vec<P<Item>> = self.handlers.iter().map(|handler| P(handler.generate(self))).collect();
        items.extend_from_slice(&[P(object_trait), P(system_struct)]);

        MacEager::items(SmallVector::many(items))
    }
}

impl HandlerInfo {
    pub fn new(name: Ident) -> HandlerInfo {
        HandlerInfo {
            name: name,
            fns: Vec::new()
        }
    }

    pub fn add_function(&mut self, function: HandlerFnInfo) {
        self.fns.push(function)
    }

    pub fn generate_as_self(&self, system: &SystemInfo) -> TraitItem {
        let args = vec![
            Arg::new_self(
                system.span,
                Mutability::Immutable,
                str_to_ident("self")
            )
        ];

        TraitItem {
            id: DUMMY_NODE_ID,
            ident: util::indent_append(str_to_ident("as_"), self.name),
            attrs: Vec::new(),
            node: TraitItemKind::Method(
                MethodSig {
                    unsafety: Unsafety::Normal,
                    constness: Constness::NotConst,
                    abi: Abi::Rust,
                    decl: P(FnDecl {
                        inputs: args,
                        output: FunctionRetTy::Ty(P(
                            util::param_ty_from_ident(
                                str_to_ident("Option"),
                                Ty {
                                    id: DUMMY_NODE_ID,
                                    node: TyKind::Rptr(
                                        None,
                                        MutTy {
                                            ty: P(util::ty_from_ident(self.name)),
                                            mutbl: Mutability::Immutable
                                        }
                                    ),
                                    span: system.span
                                }
                            )
                        )),
                        variadic: false
                    }),
                    generics: Default::default(),
                    explicit_self: respan(system.span, SelfKind::Region(
                        None,
                        Mutability::Immutable,
                        str_to_ident("self")
                    ))
                },
                None
            ),
            span: system.span
        }
    }

    pub fn generate_as_self_mut(&self, system: &SystemInfo) -> TraitItem {
        let args = vec![
            Arg::new_self(
                system.span,
                Mutability::Immutable,
                str_to_ident("self")
            )
        ];

        TraitItem {
            id: DUMMY_NODE_ID,
            ident: util::indent_append(util::indent_append(str_to_ident("as_"), self.name), str_to_ident("_mut")),
            attrs: Vec::new(),
            node: TraitItemKind::Method(
                MethodSig {
                    unsafety: Unsafety::Normal,
                    constness: Constness::NotConst,
                    abi: Abi::Rust,
                    decl: P(FnDecl {
                        inputs: args,
                        output: FunctionRetTy::Ty(P(
                            util::param_ty_from_ident(
                                str_to_ident("Option"),
                                Ty {
                                    id: DUMMY_NODE_ID,
                                    node: TyKind::Rptr(
                                        None,
                                        MutTy {
                                            ty: P(util::ty_from_ident(self.name)),
                                            mutbl: Mutability::Mutable
                                        }
                                    ),
                                    span: system.span
                                }
                            )
                        )),
                        variadic: false
                    }),
                    generics: Default::default(),
                    explicit_self: respan(system.span, SelfKind::Region(
                        None,
                        Mutability::Mutable,
                        str_to_ident("self")
                    ))
                },
                None
            ),
            span: system.span
        }
    }

    pub fn generate(&self, system: &SystemInfo) -> Item {
        Item {
            ident: self.name,
            attrs: Vec::new(),
            node: ItemKind::Trait(
                Unsafety::Normal,
                Default::default(),
                P::from_vec(Vec::new()),
                self.fns.iter().map(|function| function.generate(system)).collect()
            ),
            id: DUMMY_NODE_ID,
            span: system.span,
            vis: Visibility::Public
        }
    }
}

impl HandlerFnInfo {
    pub fn new(source: Ident, dest: Ident, args: Vec<HandlerFnArg>) -> HandlerFnInfo {
        HandlerFnInfo {
            source_name: source,
            dest_name: dest,
            args: args
        }
    }

    pub fn generate(&self, system: &SystemInfo) -> TraitItem {
        let mut args = vec![
            Arg::new_self(
                system.span,
                Mutability::Immutable,
                str_to_ident("self")
            )
        ];

        for arg in self.args.iter() {
            args.push(arg.generate(system))
        }

        TraitItem {
            id: DUMMY_NODE_ID,
            ident: self.dest_name,
            attrs: Vec::new(),
            node: TraitItemKind::Method(
                MethodSig {
                    unsafety: Unsafety::Normal,
                    constness: Constness::NotConst,
                    abi: Abi::Rust,
                    decl: P(FnDecl {
                        inputs: args,
                        output: FunctionRetTy::Default(system.span),
                        variadic: false
                    }),
                    generics: Default::default(),
                    explicit_self: respan(system.span, SelfKind::Region(
                        None,
                        Mutability::Mutable,
                        str_to_ident("self")
                    ))
                },
                None
            ),
            span: system.span
        }
    }
}

impl HandlerFnArg {
    pub fn new(name: Ident, ty: Ident) -> HandlerFnArg {
        HandlerFnArg {
            name: name,
            ty: ty
        }
    }

    pub fn generate(&self, system: &SystemInfo) -> Arg {
        Arg {
            ty: P(util::ty_from_ident(self.ty)),
            pat: P(Pat {
                id: DUMMY_NODE_ID,
                node: PatKind::Ident(
                    BindingMode::ByValue(Mutability::Immutable),
                    respan(system.span, self.name),
                    None
                ),
                span: system.span
            }),
            id: DUMMY_NODE_ID
        }
    }
}
