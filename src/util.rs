//////////////////////////////////////////////////////////////////////////////
//  File: rust-handler/util.rs
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

#![allow(dead_code)]

use syntax::ast::*;
use syntax::ptr::P;
use syntax::parse::token::{str_to_ident, InternedString};
use syntax::codemap::{respan, Spanned, DUMMY_SP};
use syntax::abi::Abi;

fn self_arg(mutability: Mutability) -> Arg {
    Arg {
        ty: P(Ty {
            id: DUMMY_NODE_ID,
            node: TyKind::Rptr(
                None,
                MutTy {
                    ty: P(Ty {
                        id: DUMMY_NODE_ID,
                        node: TyKind::ImplicitSelf,
                        span: DUMMY_SP
                    }),
                    mutbl: mutability
                }
            ),
            span: DUMMY_SP
        }),

        pat: P(Pat {
            id: DUMMY_NODE_ID,
            node: PatKind::Ident(
                BindingMode::ByValue(mutability),
                Spanned {
                    span: DUMMY_SP,
                    node: str_to_ident("self")
                },
                None
            ),
            span: DUMMY_SP
        }),

        id: DUMMY_NODE_ID
    }
}

// https://github.com/rust-lang/rust/blob/213d57983d1640d22bd69e7351731fd1adcbf9b2/src/librustc_lint/bad_style.rs#L148
fn to_snake_case(mut str: &str) -> String {
    let mut words = vec![];
    str = str.trim_left_matches(|c: char| {
        if c == '_' {
            words.push(String::new());
            true
        } else {
            false
        }
    });
    for s in str.split('_') {
        let mut last_upper = false;
        let mut buf = String::new();
        if s.is_empty() {
            continue;
        }
        for ch in s.chars() {
            if !buf.is_empty() && buf != "'"
                               && ch.is_uppercase()
                               && !last_upper {
                words.push(buf);
                buf = String::new();
            }
            last_upper = ch.is_uppercase();
            buf.extend(ch.to_lowercase());
        }
        words.push(buf);
    }
    words.join("_")
}

pub fn ident_append(a: Ident, b: Ident) -> Ident {
    let str1 = format!("{}", a.name.as_str());
    let str2 = format!("{}", b.name.as_str());
    str_to_ident(&(str1 + &str2))
}

pub fn idxs_ident(name: Ident) -> Ident {
    let mut name = format!("{}", name);
    ident_append(str_to_ident(&to_snake_case(&mut name)), str_to_ident("_idxs"))
}

pub fn as_ident(name: Ident) -> Ident {
    let mut name = format!("{}", name);
    ident_append(str_to_ident("as_"), str_to_ident(&to_snake_case(&mut name)))
}

pub fn as_mut_ident(name: Ident) -> Ident {
    ident_append(as_ident(name), str_to_ident("_mut"))
}

pub fn ty_from_ident(name: Ident) -> Ty {
    Ty {
        id: DUMMY_NODE_ID,
        node: TyKind::Path(None, Path {
            span: DUMMY_SP,
            global: false,
            segments: vec![PathSegment {
                identifier: name,
                parameters: PathParameters::none()
            }]
        }),
        span: DUMMY_SP
    }
}

pub fn ref_ty_from_ident(name: Ident) -> Ty {
    Ty {
        id: DUMMY_NODE_ID,
        node: TyKind::Rptr(
            None,
            MutTy {
                ty: P(ty_from_ident(name)),
                mutbl: Mutability::Immutable
            }
        ),
        span: DUMMY_SP
    }
}

pub fn ref_ty(ty: P<Ty>) -> Ty {
    Ty {
        id: DUMMY_NODE_ID,
        node: TyKind::Rptr(
            None,
            MutTy {
                ty: ty,
                mutbl: Mutability::Immutable
            }
        ),
        span: DUMMY_SP
    }
}

pub fn mut_ref_ty_from_ident(name: Ident) -> Ty {
    Ty {
        id: DUMMY_NODE_ID,
        node: TyKind::Rptr(
            None,
            MutTy {
                ty: P(ty_from_ident(name)),
                mutbl: Mutability::Mutable
            }
        ),
        span: DUMMY_SP
    }
}

pub fn mut_ref_ty(ty: P<Ty>) -> Ty {
    Ty {
        id: DUMMY_NODE_ID,
        node: TyKind::Rptr(
            None,
            MutTy {
                ty: ty,
                mutbl: Mutability::Mutable
            }
        ),
        span: DUMMY_SP
    }
}

pub fn param_ty_from_ident(name: Ident, ty: Ty) -> Ty {
    Ty {
        id: DUMMY_NODE_ID,
        span: DUMMY_SP,
        node: TyKind::Path(None, Path {
            span: DUMMY_SP,
            global: false,
            segments: vec![PathSegment {
                identifier: name,
                parameters: PathParameters::AngleBracketed(AngleBracketedParameterData {
                    lifetimes: Vec::new(),
                    types: P::from_vec(vec![P(ty)]),
                    bindings: P::from_vec(Vec::new())
                })
            }]
        })
    }
}

pub fn path_param_ty(names: Vec<Ident>, ty: Ty) -> Ty {
    let mut segments: Vec<PathSegment> = names.iter().map(|name| PathSegment {
        identifier: *name,
        parameters: PathParameters::none()
    }).collect();

    {
        let mut last = segments.last_mut().unwrap();
        last.parameters = PathParameters::AngleBracketed(AngleBracketedParameterData {
            lifetimes: Vec::new(),
            types: P::from_vec(vec![P(ty)]),
            bindings: P::from_vec(Vec::new())
        });
    }

    Ty {
        id: DUMMY_NODE_ID,
        span: DUMMY_SP,
        node: TyKind::Path(None, Path {
            span: DUMMY_SP,
            global: false,
            segments: segments
        })
    }
}

pub fn box_new(expr: P<Expr>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Call(
            P(Expr {
                id: DUMMY_NODE_ID,
                node: ExprKind::Path(
                    None,
                    Path {
                        span: DUMMY_SP,
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
                span: DUMMY_SP,
                attrs: None
            }),
            vec![expr]
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn vec_new() -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Call(
            P(Expr {
                id: DUMMY_NODE_ID,
                node: ExprKind::Path(
                    None,
                    Path {
                        span: DUMMY_SP,
                        global: false,
                        segments: vec![
                            PathSegment {
                                identifier: str_to_ident("Vec"),
                                parameters: PathParameters::none()
                            },
                            PathSegment {
                                identifier: str_to_ident("new"),
                                parameters: PathParameters::none()
                            }
                        ]
                    }
                ),
                span: DUMMY_SP,
                attrs: None
            }),
            Vec::new()
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_struct_field(name: Ident, ty: P<Ty>) -> StructField {
    StructField {
        span: DUMMY_SP,
        ident: Some(name),
        vis: Visibility::Inherited,
        id: DUMMY_NODE_ID,
        ty: ty,
        attrs: Vec::new()
    }
}

pub fn create_tuple_struct(name: Ident, fields: Vec<P<Ty>>) -> Item {
    Item {
        ident: name,
        attrs: Vec::new(),
        node: ItemKind::Struct(
            VariantData::Tuple(
                fields.into_iter().map(|ty| StructField {
                    span: DUMMY_SP,
                    ident: None,
                    vis: Visibility::Inherited,
                    id: DUMMY_NODE_ID,
                    ty: ty,
                    attrs: Vec::new()
                }).collect(),
                DUMMY_NODE_ID
            ),
            Default::default()
        ),
        id: DUMMY_NODE_ID,
        span: DUMMY_SP,
        vis: Visibility::Public
    }
}

pub fn create_struct(name: Ident, fields: Vec<StructField>) -> Item {
    Item {
        ident: name,
        attrs: Vec::new(),
        node: ItemKind::Struct(
            VariantData::Struct(
                fields,
                DUMMY_NODE_ID
            ),
            Default::default()
        ),
        id: DUMMY_NODE_ID,
        span: DUMMY_SP,
        vis: Visibility::Public
    }
}

pub fn create_arg(name: Ident, ty: P<Ty>) -> Arg {
    Arg {
        ty: ty,
        pat: P(Pat {
            id: DUMMY_NODE_ID,
            node: PatKind::Ident(
                BindingMode::ByValue(Mutability::Immutable),
                respan(DUMMY_SP, name),
                None
            ),
            span: DUMMY_SP
        }),
        id: DUMMY_NODE_ID
    }
}

pub fn create_mut_trait_method(name: Ident, args: Vec<Arg>, ret: Option<P<Ty>>) -> TraitItem {
    let mut args = args;
    args.insert(0, self_arg(Mutability::Mutable));

    TraitItem {
        id: DUMMY_NODE_ID,
        ident: name,
        attrs: Vec::new(),
        node: TraitItemKind::Method(
            MethodSig {
                unsafety: Unsafety::Normal,
                constness: Constness::NotConst,
                abi: Abi::Rust,
                decl: P(FnDecl {
                    inputs: args,
                    output: if let Some(ty) = ret {
                        FunctionRetTy::Ty(ty)
                    } else {
                        FunctionRetTy::Default(DUMMY_SP)
                    },
                    variadic: false
                }),
                generics: Default::default(),
            },
            None
        ),
        span: DUMMY_SP
    }
}

pub fn create_trait_method(name: Ident, args: Vec<Arg>, ret: Option<P<Ty>>) -> TraitItem {
    let mut args = args;
    args.insert(0, self_arg(Mutability::Immutable));

    TraitItem {
        id: DUMMY_NODE_ID,
        ident: name,
        attrs: Vec::new(),
        node: TraitItemKind::Method(
            MethodSig {
                unsafety: Unsafety::Normal,
                constness: Constness::NotConst,
                abi: Abi::Rust,
                decl: P(FnDecl {
                    inputs: args,
                    output: if let Some(ty) = ret {
                        FunctionRetTy::Ty(ty)
                    } else {
                        FunctionRetTy::Default(DUMMY_SP)
                    },
                    variadic: false
                }),
                generics: Default::default(),
            },
            None
        ),
        span: DUMMY_SP
    }
}

pub fn create_block_expr(block: P<Block>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Block(block),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_idx_expr(idx: P<Expr>, expr: P<Expr>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Index(expr, idx),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_var_expr(name: Ident) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Path(
            None,
            Path {
                span: DUMMY_SP,
                global: false,
                segments: vec![
                    PathSegment {
                        identifier: name,
                        parameters: PathParameters::none()
                    }
                ]
            }
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_closure_expr(args: Vec<Arg>, block: P<Block>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Closure(
            CaptureBy::Value,
            P(FnDecl {
                inputs: args,
                output: FunctionRetTy::Default(DUMMY_SP),
                variadic: false
            }),
            block,
            DUMMY_SP
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_tuple_field_expr(expr: P<Expr>, field: usize) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::TupField(
            expr,
            respan(DUMMY_SP, field)
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_num_expr(num: u64) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Lit(P(respan(DUMMY_SP, LitKind::Int(
            num,
            LitIntType::Unsuffixed
        )))),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_struct_expr(name: Ident, fields: Vec<Field>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Struct(
            Path {
                span: DUMMY_SP,
                global: false,
                segments: vec![
                    PathSegment {
                        identifier: name,
                        parameters: PathParameters::none()
                    }
                ]
            },
            fields,
            None
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_deref_expr(expr: P<Expr>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Unary(
            UnOp::Deref,
            expr
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_self_field_expr(name: Ident) -> Expr {
    create_field_expr(name, str_to_ident("self"))
}

pub fn create_field_expr(name: Ident, on: Ident) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Field(
            P(Expr {
                id: DUMMY_NODE_ID,
                node: ExprKind::Path(
                    None,
                    Path {
                        span: DUMMY_SP,
                        global: false,
                        segments: vec![
                            PathSegment {
                                identifier: on,
                                parameters: PathParameters::none()
                            }
                        ]
                    }
                ),
                span: DUMMY_SP,
                attrs: None
            }),
            respan(DUMMY_SP, name)
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_if_expr(i: P<Expr>, t: P<Block>, e: Option<P<Expr>>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::If(i, t, e),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_if_let_expr(p: P<Pat>, i: P<Expr>, t: P<Block>, e: Option<P<Expr>>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::IfLet(p, i, t, e),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_loop_expr(block: P<Block>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Loop(block, None),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_for_expr(name: Ident, range: P<Expr>, block: P<Block>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::ForLoop(
            P(Pat {
                id: DUMMY_NODE_ID,
                node: PatKind::Ident(
                    BindingMode::ByValue(Mutability::Immutable),
                    respan(DUMMY_SP, name),
                    None
                ),
                span: DUMMY_SP
            }),
            range,
            block,
            None
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_binop_expr(left: P<Expr>, op: BinOpKind, right: P<Expr>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Binary(respan(DUMMY_SP, op), left, right),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_assign_expr(left: P<Expr>, right: P<Expr>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Assign(left, right),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_assignop_expr(left: P<Expr>, op: BinOpKind, right: P<Expr>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::AssignOp(respan(DUMMY_SP, op), left, right),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn impl_method(name: Ident, args: Vec<Arg>, ret: Option<P<Ty>>, block: P<Block>) -> ImplItem {
    let mut args = args;
    args.insert(0, self_arg(Mutability::Immutable));

    ImplItem {
        id: DUMMY_NODE_ID,
        ident: name,
        vis: Visibility::Public,
        defaultness: Defaultness::Final,
        attrs: Vec::new(),
        span: DUMMY_SP,
        node: ImplItemKind::Method(
            MethodSig {
                unsafety: Unsafety::Normal,
                constness: Constness::NotConst,
                abi: Abi::Rust,
                decl: P(FnDecl {
                    inputs: args,
                    output: if let Some(ty) = ret {
                        FunctionRetTy::Ty(ty)
                    } else {
                        FunctionRetTy::Default(DUMMY_SP)
                    },
                    variadic: false
                }),
                generics: Default::default(),
            },
            block
        )
    }
}

pub fn impl_method_priv(name: Ident, args: Vec<Arg>, ret: Option<P<Ty>>, block: P<Block>) -> ImplItem {
    let mut args = args;
    args.insert(0, self_arg(Mutability::Immutable));

    ImplItem {
        id: DUMMY_NODE_ID,
        ident: name,
        vis: Visibility::Inherited,
        defaultness: Defaultness::Final,
        attrs: Vec::new(),
        span: DUMMY_SP,
        node: ImplItemKind::Method(
            MethodSig {
                unsafety: Unsafety::Normal,
                constness: Constness::NotConst,
                abi: Abi::Rust,
                decl: P(FnDecl {
                    inputs: args,
                    output: if let Some(ty) = ret {
                        FunctionRetTy::Ty(ty)
                    } else {
                        FunctionRetTy::Default(DUMMY_SP)
                    },
                    variadic: false
                }),
                generics: Default::default(),
            },
            block
        )
    }
}

pub fn impl_static_method(name: Ident, args: Vec<Arg>, ret: Option<P<Ty>>, block: P<Block>) -> ImplItem {
    ImplItem {
        id: DUMMY_NODE_ID,
        ident: name,
        vis: Visibility::Public,
        defaultness: Defaultness::Final,
        attrs: Vec::new(),
        span: DUMMY_SP,
        node: ImplItemKind::Method(
            MethodSig {
                unsafety: Unsafety::Normal,
                constness: Constness::NotConst,
                abi: Abi::Rust,
                decl: P(FnDecl {
                    inputs: args,
                    output: if let Some(ty) = ret {
                        FunctionRetTy::Ty(ty)
                    } else {
                        FunctionRetTy::Default(DUMMY_SP)
                    },
                    variadic: false
                }),
                generics: Default::default(),
            },
            block
        )
    }
}

pub fn impl_mut_method(name: Ident, args: Vec<Arg>, ret: Option<P<Ty>>, block: P<Block>) -> ImplItem {
    let mut args = args;
    args.insert(0, self_arg(Mutability::Mutable));

    ImplItem {
        id: DUMMY_NODE_ID,
        ident: name,
        vis: Visibility::Public,
        defaultness: Defaultness::Final,
        attrs: Vec::new(),
        span: DUMMY_SP,
        node: ImplItemKind::Method(
            MethodSig {
                unsafety: Unsafety::Normal,
                constness: Constness::NotConst,
                abi: Abi::Rust,
                decl: P(FnDecl {
                    inputs: args,
                    output: if let Some(ty) = ret {
                        FunctionRetTy::Ty(ty)
                    } else {
                        FunctionRetTy::Default(DUMMY_SP)
                    },
                    variadic: false
                }),
                generics: Default::default(),
            },
            block
        )
    }
}

pub fn impl_mut_method_priv(name: Ident, args: Vec<Arg>, ret: Option<P<Ty>>, block: P<Block>) -> ImplItem {
    let mut args = args;
    args.insert(0, self_arg(Mutability::Mutable));

    ImplItem {
        id: DUMMY_NODE_ID,
        ident: name,
        vis: Visibility::Inherited,
        defaultness: Defaultness::Final,
        attrs: Vec::new(),
        span: DUMMY_SP,
        node: ImplItemKind::Method(
            MethodSig {
                unsafety: Unsafety::Normal,
                constness: Constness::NotConst,
                abi: Abi::Rust,
                decl: P(FnDecl {
                    inputs: args,
                    output: if let Some(ty) = ret {
                        FunctionRetTy::Ty(ty)
                    } else {
                        FunctionRetTy::Default(DUMMY_SP)
                    },
                    variadic: false
                }),
                generics: Default::default(),
            },
            block
        )
    }
}

pub fn create_cast_expr(expr: P<Expr>, ty: P<Ty>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Cast(
            expr,
            ty
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_method_call(name: Ident, on: P<Expr>, args: Vec<P<Expr>>) -> Expr {
    let mut args = args;
    args.insert(0, on);

    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::MethodCall(
            respan(DUMMY_SP, name),
            Vec::new(),
            args
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_call(expr: P<Expr>, args: Vec<P<Expr>>) -> Expr {
    Expr {
        id: DUMMY_NODE_ID,
        node: ExprKind::Call(
            expr,
            args
        ),
        span: DUMMY_SP,
        attrs: None
    }
}

pub fn create_block(stmts: Vec<Stmt>, expr: Option<P<Expr>>) -> Block {
    Block {
        stmts: stmts,
        expr: expr,
        id: DUMMY_NODE_ID,
        rules: BlockCheckMode::Default,
        span: DUMMY_SP
    }
}

pub fn create_unsafe_block(stmts: Vec<Stmt>, expr: Option<P<Expr>>) -> Block {
    Block {
        stmts: stmts,
        expr: expr,
        id: DUMMY_NODE_ID,
        rules: BlockCheckMode::Unsafe(UnsafeSource::CompilerGenerated),
        span: DUMMY_SP
    }
}

pub fn create_return_block(expr: Option<P<Expr>>) -> Block {
    create_block(
        vec![create_stmt(P(Expr {
            id: DUMMY_NODE_ID,
            node: ExprKind::Ret(expr),
            span: DUMMY_SP,
            attrs: None
        }))],
        None
    )
}

pub fn create_impl(name: Ident, tr: Option<Ident>, items: Vec<ImplItem>) -> Item {
    Item {
        ident: name,
        attrs: Vec::new(),
        node: ItemKind::Impl(
            Unsafety::Normal,
            ImplPolarity::Positive,
            Default::default(),
            tr.map(|name| TraitRef {
                path: Path {
                    span: DUMMY_SP,
                    global: false,
                    segments: vec![
                        PathSegment {
                            identifier: name,
                            parameters: PathParameters::none()
                        }
                    ]
                },
                ref_id: DUMMY_NODE_ID
            }),
            P(ty_from_ident(name)),
            items
        ),
        id: DUMMY_NODE_ID,
        span: DUMMY_SP,
        vis: Visibility::Inherited
    }
}

pub fn create_trait(name: Ident, reqs: &Vec<Ident>, items: &Vec<TraitItem>) -> Item {
    Item {
        ident: name,
        attrs: Vec::new(),
        node: ItemKind::Trait(
            Unsafety::Normal,
            Default::default(),
            P::from_vec(reqs.iter().map(|req| TyParamBound::TraitTyParamBound(
                PolyTraitRef {
                    bound_lifetimes: Vec::new(),
                    trait_ref: TraitRef {
                        path: Path {
                            span: DUMMY_SP,
                            global: false,
                            segments: vec![
                                PathSegment {
                                    identifier: *req,
                                    parameters: PathParameters::none()
                                }
                            ]
                        },
                        ref_id: DUMMY_NODE_ID
                    },
                    span: DUMMY_SP
                },
                TraitBoundModifier::None
            )).collect()),
            items.clone()
        ),
        id: DUMMY_NODE_ID,
        span: DUMMY_SP,
        vis: Visibility::Public
    }
}

pub fn create_field(name: Ident, value: P<Expr>) -> Field {
    Field {
        ident: respan(DUMMY_SP, name),
        expr: value,
        span: DUMMY_SP
    }
}

pub fn create_let_mut_stmt(name: Ident, expr: Option<P<Expr>>) -> Stmt {
    respan(DUMMY_SP, StmtKind::Decl(
        P(respan(DUMMY_SP, DeclKind::Local(
            P(Local {
                pat: P(Pat {
                    id: DUMMY_NODE_ID,
                    node: PatKind::Ident(
                        BindingMode::ByValue(Mutability::Mutable),
                        respan(DUMMY_SP, name),
                        None
                    ),
                    span: DUMMY_SP
                }),
                ty: None,
                init: expr,
                id: DUMMY_NODE_ID,
                span: DUMMY_SP,
                attrs: None
            })
        ))),
        DUMMY_NODE_ID
    ))
}

pub fn create_let_stmt(name: Ident, expr: Option<P<Expr>>) -> Stmt {
    respan(DUMMY_SP, StmtKind::Decl(
        P(respan(DUMMY_SP, DeclKind::Local(
            P(Local {
                pat: P(Pat {
                    id: DUMMY_NODE_ID,
                    node: PatKind::Ident(
                        BindingMode::ByValue(Mutability::Immutable),
                        respan(DUMMY_SP, name),
                        None
                    ),
                    span: DUMMY_SP
                }),
                ty: None,
                init: expr,
                id: DUMMY_NODE_ID,
                span: DUMMY_SP,
                attrs: None
            })
        ))),
        DUMMY_NODE_ID
    ))
}

pub fn create_stmt(expr: P<Expr>) -> Stmt {
    respan(DUMMY_SP, StmtKind::Semi(expr, DUMMY_NODE_ID))
}

pub fn create_tuple_struct_pat(name: Ident, bindings: Vec<Ident>) -> Pat {
    Pat {
        id: DUMMY_NODE_ID,
        node: PatKind::TupleStruct(
            Path {
                span: DUMMY_SP,
                global: false,
                segments: vec![PathSegment {
                    identifier: name,
                    parameters: PathParameters::none()
                }]
            },
            bindings.into_iter().map(|ident| P(Pat {
                id: DUMMY_NODE_ID,
                node: PatKind::Ident(
                    BindingMode::ByValue(Mutability::Immutable),
                    respan(DUMMY_SP, ident),
                    None
                ),
                span: DUMMY_SP
            })).collect(),
            None
        ),
        span: DUMMY_SP
    }
}

pub fn create_derive(items: Vec<InternedString>) -> Attribute {
    respan(DUMMY_SP, Attribute_ {
        id: AttrId(0),
        style: AttrStyle::Outer,
        value: P(respan(DUMMY_SP, MetaItemKind::List(
            InternedString::new("derive"),
            items.into_iter().map(|item| P(respan(DUMMY_SP, MetaItemKind::Word(item)))).collect()
        ))),
        is_sugared_doc: false
    })
}
