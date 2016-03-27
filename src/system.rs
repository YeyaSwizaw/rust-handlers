//////////////////////////////////////////////////////////////////////////////
//  File: rust-handler/system.rs
//////////////////////////////////////////////////////////////////////////////
//  Copyright 2016 Samuel Sleight
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//////////////////////////////////////////////////////////////////////////////

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
        util::ident_append(self.name, str_to_ident("Object"))
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
        let objects_field = util::create_struct_field(
            str_to_ident("objects"), 
            P(util::param_ty_from_ident(
                str_to_ident("Vec"),
                util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                )
            ))
        );

        let mut fields = vec![objects_field];

        for handler in self.handlers.iter() {
            fields.push(util::create_struct_field(
                util::idxs_ident(handler.name),
                P(util::param_ty_from_ident(
                    str_to_ident("Vec"),
                    util::ty_from_ident(str_to_ident("usize"))
                ))
            ));
        }

        util::create_struct(self.name, fields)
    }

    fn generate_fn_new_impl(&self) -> ImplItem {
        let mut fields = vec![
            Field {
                ident: respan(self.span, str_to_ident("objects")),
                expr: P(util::vec_new()),
                span: self.span
            }
        ];

        for handler in self.handlers.iter() {
            fields.push(Field {
                ident: respan(self.span, util::idxs_ident(handler.name)),
                expr: P(util::vec_new()),
                span: self.span
            });
        }

        ImplItem {
            id: DUMMY_NODE_ID,
            ident: str_to_ident("new"),
            vis: Visibility::Public,
            defaultness: Defaultness::Final,
            attrs: Vec::new(),
            span: self.span,
            node: ImplItemKind::Method(
                MethodSig {
                    unsafety: Unsafety::Normal,
                    constness: Constness::NotConst,
                    abi: Abi::Rust,
                    decl: P(FnDecl {
                        inputs: Vec::new(),
                        output: FunctionRetTy::Ty(P(util::ty_from_ident(self.name))),
                        variadic: false
                    }),
                    generics: Default::default(),
                    explicit_self: respan(self.span, SelfKind::Static)
                },

                P(Block {
                    stmts: Vec::new(),
                    expr: Some(P(Expr {
                        id: DUMMY_NODE_ID,
                        node: ExprKind::Struct(
                            Path {
                                span: self.span,
                                global: false,
                                segments: vec![
                                    PathSegment {
                                        identifier: self.name,
                                        parameters: PathParameters::none()
                                    }
                                ]
                            },
                            fields,
                            None
                        ),
                        span: self.span,
                        attrs: None
                    })),
                    id: DUMMY_NODE_ID,
                    rules: BlockCheckMode::Default,
                    span: self.span
                })
            )
        }
    }

    fn generate_fn_add_impl(&self) -> ImplItem {
        let mut stmts = vec![
            respan(self.span, StmtKind::Decl(
                P(respan(self.span, DeclKind::Local(
                    P(Local {
                        pat: P(Pat {
                            id: DUMMY_NODE_ID,
                            node: PatKind::Ident(
                                BindingMode::ByValue(Mutability::Immutable),
                                respan(self.span, str_to_ident("idx")),
                                None
                            ),
                            span: self.span
                        }),
                        ty: None,
                        init: Some(P(Expr {
                            id: DUMMY_NODE_ID,
                            node: ExprKind::MethodCall(
                                respan(self.span, str_to_ident("len")),
                                Vec::new(),
                                vec![P(Expr {
                                    id: DUMMY_NODE_ID,
                                    node: ExprKind::Field(
                                        P(Expr {
                                            id: DUMMY_NODE_ID,
                                            node: ExprKind::Path(
                                                None,
                                                Path {
                                                    span: self.span,
                                                    global: false,
                                                    segments: vec![
                                                        PathSegment {
                                                            identifier: str_to_ident("self"),
                                                            parameters: PathParameters::none()
                                                        }
                                                    ]
                                                }
                                            ),
                                            span: self.span,
                                            attrs: None
                                        }),
                                        respan(self.span, str_to_ident("objects"))
                                    ),
                                    span: self.span,
                                    attrs: None
                                })],
                            ),
                            span: self.span,
                            attrs: None
                        })),
                        id: DUMMY_NODE_ID,
                        span: self.span,
                        attrs: None
                    })
                ))),
                DUMMY_NODE_ID
            )),

            respan(self.span, StmtKind::Semi(
                P(Expr {
                    id: DUMMY_NODE_ID,
                    node: ExprKind::MethodCall(
                        respan(self.span, str_to_ident("push")),
                        Vec::new(),
                        vec![
                            P(Expr {
                                id: DUMMY_NODE_ID,
                                node: ExprKind::Field(
                                    P(Expr {
                                        id: DUMMY_NODE_ID,
                                        node: ExprKind::Path(
                                            None,
                                            Path {
                                                span: self.span,
                                                global: false,
                                                segments: vec![
                                                    PathSegment {
                                                        identifier: str_to_ident("self"),
                                                        parameters: PathParameters::none()
                                                    }
                                                ]
                                            }
                                        ),
                                        span: self.span,
                                        attrs: None
                                    }),
                                    respan(self.span, str_to_ident("objects"))
                                ),
                                span: self.span,
                                attrs: None
                            }),

                            P(Expr {
                                id: DUMMY_NODE_ID,
                                node: ExprKind::Call(
                                    P(Expr {
                                        id: DUMMY_NODE_ID,
                                        node: ExprKind::Path(
                                            None,
                                            Path {
                                                span: self.span,
                                                global: false,
                                                segments: vec![
                                                    PathSegment {
                                                        identifier: str_to_ident("Box"),
                                                        parameters: PathParameters::none()
                                                    },
                                                    PathSegment {
                                                        identifier: str_to_ident("new"),
                                                        parameters: PathParameters::none()
                                                    }
                                                ]
                                            }
                                        ),
                                        span: self.span,
                                        attrs: None
                                    }),
                                    vec![P(Expr {
                                        id: DUMMY_NODE_ID,
                                        node: ExprKind::Path(
                                            None,
                                            Path {
                                                span: self.span,
                                                global: false,
                                                segments: vec![
                                                    PathSegment {
                                                        identifier: str_to_ident("object"),
                                                        parameters: PathParameters::none()
                                                    }
                                                ]
                                            }
                                        ),
                                        span: self.span,
                                        attrs: None
                                    })]
                                ),
                                span: self.span,
                                attrs: None
                            })
                        ]
                    ),
                    span: self.span,
                    attrs: None
                }),
                DUMMY_NODE_ID
            )),

            respan(self.span, StmtKind::Decl(
                P(respan(self.span, DeclKind::Local(
                    P(Local {
                        pat: P(Pat {
                            id: DUMMY_NODE_ID,
                            node: PatKind::Ident(
                                BindingMode::ByValue(Mutability::Immutable),
                                respan(self.span, str_to_ident("object")),
                                None
                            ),
                            span: self.span
                        }),
                        ty: None,
                        init: Some(P(Expr {
                            id: DUMMY_NODE_ID,
                            node: ExprKind::MethodCall(
                                respan(self.span, str_to_ident("unwrap")),
                                Vec::new(),
                                vec![P(Expr {
                                    id: DUMMY_NODE_ID,
                                    node: ExprKind::MethodCall(
                                        respan(self.span, str_to_ident("last")),
                                        Vec::new(),
                                        vec![P(Expr {
                                            id: DUMMY_NODE_ID,
                                            node: ExprKind::Field(
                                                P(Expr {
                                                    id: DUMMY_NODE_ID,
                                                    node: ExprKind::Path(
                                                        None,
                                                        Path {
                                                            span: self.span,
                                                            global: false,
                                                            segments: vec![
                                                                PathSegment {
                                                                    identifier: str_to_ident("self"),
                                                                    parameters: PathParameters::none()
                                                                }
                                                            ]
                                                        }
                                                    ),
                                                    span: self.span,
                                                    attrs: None
                                                }),
                                                respan(self.span, str_to_ident("objects"))
                                            ),
                                            span: self.span,
                                            attrs: None
                                        })]
                                    ),
                                    span: self.span,
                                    attrs: None
                                })],
                            ),
                            span: self.span,
                            attrs: None
                        })),
                        id: DUMMY_NODE_ID,
                        span: self.span,
                        attrs: None
                    })
                ))),
                DUMMY_NODE_ID
            )),
        ];

        for handler in self.handlers.iter() {
            stmts.push(respan(self.span, StmtKind::Semi(P(handler.generate_add_check()), DUMMY_NODE_ID)));
        }

        ImplItem {
            id: DUMMY_NODE_ID,
            ident: str_to_ident("add"),
            vis: Visibility::Public,
            defaultness: Defaultness::Final,
            attrs: Vec::new(),
            span: self.span,
            node: ImplItemKind::Method(
                MethodSig {
                    unsafety: Unsafety::Normal,
                    constness: Constness::NotConst,
                    abi: Abi::Rust,
                    decl: P(FnDecl {
                        inputs: vec![
                            Arg::new_self(
                                self.span,
                                Mutability::Immutable,
                                str_to_ident("self")
                            ),
                            Arg {
                                ty: P(util::ty_from_ident(str_to_ident("O"))),
                                pat: P(Pat {
                                    id: DUMMY_NODE_ID,
                                    node: PatKind::Ident(
                                        BindingMode::ByValue(Mutability::Immutable),
                                        respan(self.span, str_to_ident("object")),
                                        None
                                    ),
                                    span: self.span
                                }),
                                id: DUMMY_NODE_ID
                            }
                        ],
                        output: FunctionRetTy::Default(self.span),
                        variadic: false
                    }),
                    generics: Generics {
                        lifetimes: Vec::new(),
                        ty_params: P::from_vec(vec![
                            TyParam {
                                ident: str_to_ident("O"),
                                id: DUMMY_NODE_ID,
                                bounds: P::from_vec(Vec::new()),
                                default: None,
                                span: self.span
                            }
                        ]),
                        where_clause: WhereClause {
                            id: DUMMY_NODE_ID,
                            predicates: vec![
                                WherePredicate::BoundPredicate(WhereBoundPredicate {
                                    span: self.span,
                                    bound_lifetimes: Vec::new(),
                                    bounded_ty: P(util::ty_from_ident(str_to_ident("O"))),
                                    bounds: P::from_vec(vec![
                                        TyParamBound::RegionTyParamBound(
                                            Lifetime {
                                                id: DUMMY_NODE_ID,
                                                span: self.span,
                                                name: str_to_ident("'static").name
                                            }
                                        ),
                                        TyParamBound::TraitTyParamBound(
                                            PolyTraitRef {
                                                bound_lifetimes: Vec::new(),
                                                trait_ref: TraitRef {
                                                    path: Path {
                                                        span: self.span,
                                                        global: false,
                                                        segments: vec![
                                                            PathSegment {
                                                                identifier: self.object_name(),
                                                                parameters: PathParameters::none()
                                                            }
                                                        ]
                                                    },
                                                    ref_id: DUMMY_NODE_ID
                                                },
                                                span: self.span
                                            },
                                            TraitBoundModifier::None
                                        )
                                    ])
                                })
                            ]
                        }
                    },
                    explicit_self: respan(self.span, SelfKind::Region(
                        None,
                        Mutability::Mutable,
                        str_to_ident("self")
                    ))
                },

                P(Block {
                    stmts: stmts,
                    expr: None,
                    id: DUMMY_NODE_ID,
                    rules: BlockCheckMode::Default,
                    span: self.span
                })
            )
        }
    }

    fn generate_impl(&self) -> Item {
        let mut fns = vec![
            self.generate_fn_new_impl(),
            self.generate_fn_add_impl()
        ];

        for handler in self.handlers.iter() {
            handler.generate_signal_impl(self, &mut fns);
        }

        Item {
            ident: self.name,
            attrs: Vec::new(),
            node: ItemKind::Impl(
                Unsafety::Normal,
                ImplPolarity::Positive,
                Default::default(),
                None,
                P(util::ty_from_ident(self.name)),
                fns
            ),
            id: DUMMY_NODE_ID,
            span: self.span,
            vis: Visibility::Inherited
        }
    }

    pub fn generate_object_impl(&self, thing: Ident, impls: &Vec<String>) -> Box<MacResult> {
        let mut items = Vec::new();

        for handler in self.handlers.iter() {
            items.extend_from_slice(&[
                ImplItem {
                    id: DUMMY_NODE_ID,
                    ident: util::as_ident(handler.name),
                    vis: Visibility::Inherited,
                    defaultness: Defaultness::Final,
                    attrs: Vec::new(),
                    span: self.span,
                    node: ImplItemKind::Method(
                        MethodSig {
                            unsafety: Unsafety::Normal,
                            constness: Constness::NotConst,
                            abi: Abi::Rust,
                            decl: P(FnDecl {
                                inputs: vec![
                                    Arg::new_self(
                                        self.span,
                                        Mutability::Immutable,
                                        str_to_ident("self")
                                    )
                                ],
                                output: FunctionRetTy::Ty(P(
                                    util::param_ty_from_ident(
                                        str_to_ident("Option"),
                                        Ty {
                                            id: DUMMY_NODE_ID,
                                            node: TyKind::Rptr(
                                                None,
                                                MutTy {
                                                    ty: P(util::ty_from_ident(handler.name)),
                                                    mutbl: Mutability::Immutable
                                                }
                                            ),
                                            span: self.span
                                        }
                                    )
                                )),
                                variadic: false
                            }),
                            generics: Default::default(),
                            explicit_self: respan(self.span, SelfKind::Region(
                                None,
                                Mutability::Immutable,
                                str_to_ident("self")
                            ))
                        },

                        P(Block {
                            stmts: Vec::new(),
                            expr: Some(P(Expr {
                                id: DUMMY_NODE_ID,
                                node: if impls.contains(&format!("{}", handler.name)) {
                                    ExprKind::Call(
                                        P(Expr {
                                            id: DUMMY_NODE_ID,
                                            node: ExprKind::Path(
                                                None,
                                                Path {
                                                    span: self.span,
                                                    global: false,
                                                    segments: vec![
                                                        PathSegment {
                                                            identifier: str_to_ident("Some"),
                                                            parameters: PathParameters::none()
                                                        }
                                                    ]
                                                }
                                            ),
                                            span: self.span,
                                            attrs: None
                                        }),
                                        vec![P(Expr {
                                            id: DUMMY_NODE_ID,
                                            node: ExprKind::Cast(
                                                P(Expr {
                                                    id: DUMMY_NODE_ID,
                                                    node: ExprKind::Path(
                                                        None,
                                                        Path {
                                                            span: self.span,
                                                            global: false,
                                                            segments: vec![
                                                                PathSegment {
                                                                    identifier: str_to_ident("self"),
                                                                    parameters: PathParameters::none()
                                                                }
                                                            ]
                                                        }
                                                    ),
                                                    span: self.span,
                                                    attrs: None
                                                }),
                                                P(Ty {
                                                    id: DUMMY_NODE_ID,
                                                    node: TyKind::Rptr(
                                                        None,
                                                        MutTy {
                                                            ty: P(util::ty_from_ident(handler.name)),
                                                            mutbl: Mutability::Immutable
                                                        }
                                                    ),
                                                    span: self.span
                                                })
                                            ),
                                            span: self.span,
                                            attrs: None
                                        })]
                                    )
                                } else {
                                    ExprKind::Path(
                                        None,
                                        Path {
                                            span: self.span,
                                            global: false,
                                            segments: vec![
                                                PathSegment {
                                                    identifier: str_to_ident("None"),
                                                    parameters: PathParameters::none()
                                                }
                                            ]
                                        }
                                    )
                                },
                                span: self.span,
                                attrs: None
                            })),
                            id: DUMMY_NODE_ID,
                            rules: BlockCheckMode::Default,
                            span: self.span
                        })
                    )
                },

                ImplItem {
                    id: DUMMY_NODE_ID,
                    ident: util::as_mut_ident(handler.name),
                    vis: Visibility::Inherited,
                    defaultness: Defaultness::Final,
                    attrs: Vec::new(),
                    span: self.span,
                    node: ImplItemKind::Method(
                        MethodSig {
                            unsafety: Unsafety::Normal,
                            constness: Constness::NotConst,
                            abi: Abi::Rust,
                            decl: P(FnDecl {
                                inputs: vec![
                                    Arg::new_self(
                                        self.span,
                                        Mutability::Immutable,
                                        str_to_ident("self")
                                    )
                                ],
                                output: FunctionRetTy::Ty(P(
                                    util::param_ty_from_ident(
                                        str_to_ident("Option"),
                                        Ty {
                                            id: DUMMY_NODE_ID,
                                            node: TyKind::Rptr(
                                                None,
                                                MutTy {
                                                    ty: P(util::ty_from_ident(handler.name)),
                                                    mutbl: Mutability::Mutable
                                                }
                                            ),
                                            span: self.span
                                        }
                                    )
                                )),
                                variadic: false
                            }),
                            generics: Default::default(),
                            explicit_self: respan(self.span, SelfKind::Region(
                                None,
                                Mutability::Mutable,
                                str_to_ident("self")
                            ))
                        },

                        P(Block {
                            stmts: Vec::new(),
                            expr: Some(P(Expr {
                                id: DUMMY_NODE_ID,
                                node: if impls.contains(&format!("{}", handler.name)) {
                                    ExprKind::Call(
                                        P(Expr {
                                            id: DUMMY_NODE_ID,
                                            node: ExprKind::Path(
                                                None,
                                                Path {
                                                    span: self.span,
                                                    global: false,
                                                    segments: vec![
                                                        PathSegment {
                                                            identifier: str_to_ident("Some"),
                                                            parameters: PathParameters::none()
                                                        }
                                                    ]
                                                }
                                            ),
                                            span: self.span,
                                            attrs: None
                                        }),
                                        vec![P(Expr {
                                            id: DUMMY_NODE_ID,
                                            node: ExprKind::Cast(
                                                P(Expr {
                                                    id: DUMMY_NODE_ID,
                                                    node: ExprKind::Path(
                                                        None,
                                                        Path {
                                                            span: self.span,
                                                            global: false,
                                                            segments: vec![
                                                                PathSegment {
                                                                    identifier: str_to_ident("self"),
                                                                    parameters: PathParameters::none()
                                                                }
                                                            ]
                                                        }
                                                    ),
                                                    span: self.span,
                                                    attrs: None
                                                }),
                                                P(Ty {
                                                    id: DUMMY_NODE_ID,
                                                    node: TyKind::Rptr(
                                                        None,
                                                        MutTy {
                                                            ty: P(util::ty_from_ident(handler.name)),
                                                            mutbl: Mutability::Mutable
                                                        }
                                                    ),
                                                    span: self.span
                                                })
                                            ),
                                            span: self.span,
                                            attrs: None
                                        })]
                                    )
                                } else {
                                    ExprKind::Path(
                                        None,
                                        Path {
                                            span: self.span,
                                            global: false,
                                            segments: vec![
                                                PathSegment {
                                                    identifier: str_to_ident("None"),
                                                    parameters: PathParameters::none()
                                                }
                                            ]
                                        }
                                    )
                                },
                                span: self.span,
                                attrs: None
                            })),
                            id: DUMMY_NODE_ID,
                            rules: BlockCheckMode::Default,
                            span: self.span
                        })
                    )
                },
            ]);
        }

        MacEager::items(SmallVector::one(P(Item {
            ident: thing,
            attrs: Vec::new(),
            node: ItemKind::Impl(
                Unsafety::Normal,
                ImplPolarity::Positive,
                Default::default(),
                Some(TraitRef {
                    path: Path {
                        span: self.span,
                        global: false,
                        segments: vec![
                            PathSegment {
                                identifier: self.object_name(),
                                parameters: PathParameters::none()
                            }
                        ]
                    },
                    ref_id: DUMMY_NODE_ID
                }),
                P(util::ty_from_ident(thing)),
                items
            ),
            id: DUMMY_NODE_ID,
            span: self.span,
            vis: Visibility::Inherited
        })))
    }

    pub fn generate_ast(&self) -> Box<MacResult> {
        let object_trait = self.generate_object_trait();
        let system_struct = self.generate_struct();
        let struct_impl = self.generate_impl();

        let mut items: Vec<P<Item>> = self.handlers.iter().map(|handler| P(handler.generate(self))).collect();
        items.extend_from_slice(&[P(object_trait), P(system_struct), P(struct_impl)]);

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
            ident: util::as_ident(self.name),
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
            ident: util::as_mut_ident(self.name),
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
                self.fns.iter().map(|function| function.generate()).collect()
            ),
            id: DUMMY_NODE_ID,
            span: system.span,
            vis: Visibility::Public
        }
    }

    pub fn generate_signal_impl(&self, system: &SystemInfo, items: &mut Vec<ImplItem>) {
        for func in self.fns.iter() {
            let mut args = vec![
                Arg::new_self(
                    system.span,
                    Mutability::Immutable,
                    str_to_ident("self")
                )
            ];

            let mut dest_args = vec![
                P(Expr {
                    id: DUMMY_NODE_ID,
                    node: ExprKind::MethodCall(
                        respan(system.span, str_to_ident("unwrap")),
                        Vec::new(),
                        vec![
                            P(Expr {
                                id: DUMMY_NODE_ID,
                                node: ExprKind::MethodCall(
                                    respan(system.span, util::as_mut_ident(self.name)),
                                    Vec::new(),
                                    vec![
                                        P(Expr {
                                            id: DUMMY_NODE_ID,
                                            node: ExprKind::MethodCall(
                                                respan(system.span, str_to_ident("get_unchecked_mut")),
                                                Vec::new(),
                                                vec![
                                                    P(Expr {
                                                        id: DUMMY_NODE_ID,
                                                        node: ExprKind::Field(
                                                            P(Expr {
                                                                id: DUMMY_NODE_ID,
                                                                node: ExprKind::Path(
                                                                    None,
                                                                    Path {
                                                                        span: system.span,
                                                                        global: false,
                                                                        segments: vec![
                                                                            PathSegment {
                                                                                identifier: str_to_ident("self"),
                                                                                parameters: PathParameters::none()
                                                                            }
                                                                        ]
                                                                    }
                                                                ),
                                                                span: system.span,
                                                                attrs: None
                                                            }),
                                                            respan(system.span, str_to_ident("objects"))
                                                        ),
                                                        span: system.span,
                                                        attrs: None
                                                    }),

                                                    P(Expr {
                                                        id: DUMMY_NODE_ID,
                                                        node: ExprKind::Unary(
                                                            UnOp::Deref,
                                                            P(Expr {
                                                                id: DUMMY_NODE_ID,
                                                                node: ExprKind::Path(
                                                                    None,
                                                                    Path {
                                                                        span: system.span,
                                                                        global: false,
                                                                        segments: vec![
                                                                            PathSegment {
                                                                                identifier: str_to_ident("idx"),
                                                                                parameters: PathParameters::none()
                                                                            }
                                                                        ]
                                                                    }
                                                                ),
                                                                span: system.span,
                                                                attrs: None
                                                            })
                                                        ),
                                                        span: system.span,
                                                        attrs: None
                                                    })
                                                ]
                                            ),
                                            span: system.span,
                                            attrs: None
                                        })
                                    ]
                                ),
                                span: system.span,
                                attrs: None
                            })
                        ]
                    ),
                    span: system.span,
                    attrs: None
                })
            ];


            for arg in func.args.iter() {
                let a = arg.generate();
                args.push(a);
                dest_args.push(P(Expr {
                    id: DUMMY_NODE_ID,
                    node: ExprKind::Path(
                        None,
                        Path {
                            span: system.span,
                            global: false,
                            segments: vec![
                                PathSegment {
                                    identifier: arg.name,
                                    parameters: PathParameters::none()
                                }
                            ]
                        }
                    ),
                    span: system.span,
                    attrs: None
                }));
            }

            items.push(ImplItem {
                id: DUMMY_NODE_ID,
                ident: func.source_name,
                vis: Visibility::Public,
                defaultness: Defaultness::Final,
                attrs: Vec::new(),
                span: system.span,
                node: ImplItemKind::Method(
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
                        )),
                    },

                    P(util::create_block(
                        vec![
                            P(util::create_for_expr(
                                str_to_ident("idx"),
                                P(util::create_method_call(
                                    str_to_ident("iter"),
                                    P(util::create_self_field_expr(util::idxs_ident(self.name))),
                                    Vec::new()
                                )),
                                P(util::create_unsafe_block(
                                    vec![
                                        P(util::create_method_call(
                                            func.dest_name,
                                            dest_args[0].clone(),
                                            dest_args[1..].to_vec()
                                        ))
                                    ],
                                    None
                                ))
                            ))
                        ],
                        None
                    ))
                )
            });
        }
    }

    pub fn generate_add_check(&self) -> Expr {
        util::create_if_expr(
            P(util::create_method_call(
                str_to_ident("is_some"),
                P(util::create_method_call(
                    util::as_ident(self.name),
                    P(util::create_var_expr(str_to_ident("object"))),
                    Vec::new()
                )),
                Vec::new()
            )),

            P(util::create_block(
                vec![
                    P(util::create_method_call(
                        str_to_ident("push"),
                        P(util::create_self_field_expr(util::idxs_ident(self.name))),
                        vec![
                            P(util::create_var_expr(str_to_ident("idx")))
                        ]
                    )),
                ],
                None
            ))
        )
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

    pub fn generate(&self) -> TraitItem {
        util::create_mut_trait_method(
            self.dest_name,
            self.args.iter().map(|arg| arg.generate()).collect()
        )
    }
}

impl HandlerFnArg {
    pub fn new(name: Ident, ty: Ident) -> HandlerFnArg {
        HandlerFnArg {
            name: name,
            ty: ty
        }
    }

    pub fn generate(&self) -> Arg {
        util::create_arg(self.name, P(util::ty_from_ident(self.ty)))
    }
}
