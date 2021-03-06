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
use syntax::codemap::Span;
use syntax::ext::base::{MacResult, MacEager};
use syntax::util::small_vector::SmallVector;
use syntax::parse::token::{str_to_ident, InternedString};

use ::util;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub name: Ident,
    pub span: Span,
    pub reqs: Vec<Ident>,
    pub handlers: Vec<HandlerInfo>
}

#[derive(Debug, Clone)]
pub struct HandlerInfo {
    pub name: Ident,
    pub reqs: Vec<Ident>,
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
    pub ptr: Option<Mutability>
}

impl SystemInfo {
    pub fn new(name: Ident, span: Span) -> SystemInfo {
        SystemInfo {
            name: name,
            span: span,
            reqs: Vec::new(),
            handlers: Vec::new()
        }
    }

    pub fn add_requirement(&mut self, req: Ident) {
        self.reqs.push(req);
    }

    pub fn add_handler(&mut self, handler: HandlerInfo) {
        self.handlers.push(handler);
    }

    fn object_name(&self) -> Ident {
        util::ident_append(self.name, str_to_ident("Object"))
    }

    fn idx_name(&self) -> Ident {
        util::ident_append(self.name, str_to_ident("Index"))
    }

    fn generate_object_trait(&self) -> Item {
        let mut fns = Vec::new();

        for handler in self.handlers.iter() {
            fns.push(handler.generate_as_self());
            fns.push(handler.generate_as_self_mut());
        }

        util::create_trait(
            self.object_name(),
            &self.reqs,
            &fns
        )
    }

    fn generate_idx_struct(&self) -> Item {
        let mut item = util::create_tuple_struct(
            self.idx_name(),
            vec![P(util::ty_from_ident(str_to_ident("usize")))]
        );

        item.attrs = vec![util::create_derive(vec![
            InternedString::new("Copy"),
            InternedString::new("Clone"),
            InternedString::new("Eq"),
            InternedString::new("PartialEq"),
        ])];

        item
    }

    fn generate_struct(&self) -> Item {
        let mut fields = vec![
            util::create_struct_field(
                str_to_ident("objects"), 
                P(util::param_ty_from_ident(
                    str_to_ident("Vec"),
                    util::param_ty_from_ident(
                        str_to_ident("Box"),
                        util::ty_from_ident(self.object_name())
                    )
                ))
            ),

            util::create_struct_field(
                str_to_ident("idxs"), 
                P(util::param_ty_from_ident(
                    str_to_ident("Vec"),
                    util::param_ty_from_ident(
                        str_to_ident("Option"),
                        util::ty_from_ident(str_to_ident("usize")),
                    )
                ))
            ),
        ];

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
            util::create_field(
                str_to_ident("objects"),
                P(util::vec_new())
            ),
            util::create_field(
                str_to_ident("idxs"),
                P(util::vec_new())
            ),
        ];

        for handler in self.handlers.iter() {
            fields.push(util::create_field(
                util::idxs_ident(handler.name),
                P(util::vec_new())
            ));
        }

        util::impl_static_method(
            str_to_ident("new"),
            Vec::new(),
            Some(P(util::ty_from_ident(self.name))),
            P(util::create_block(
                Vec::new(),
                Some(P(util::create_struct_expr(self.name, fields)))
            ))
        )
    }

    fn generate_fn_add_impl(&self) -> ImplItem {
        let mut stmts = vec![
            // let idx = self.idxs.len();
            util::create_let_stmt(
                str_to_ident("idx"),
                Some(P(util::create_method_call(
                    str_to_ident("len"),
                    P(util::create_self_field_expr(str_to_ident("idxs"))),
                    Vec::new()
                )))
            ),

            // self.idxs.push(Some(self.objects.len()));
            util::create_stmt(P(util::create_method_call(
                str_to_ident("push"),
                P(util::create_self_field_expr(str_to_ident("idxs"))),
                vec![P(util::create_call(
                    P(util::create_var_expr(str_to_ident("Some"))),
                    vec![P(util::create_method_call(
                        str_to_ident("len"),
                        P(util::create_self_field_expr(str_to_ident("objects"))),
                        Vec::new()
                    ))]
                ))]
            ))),

            // self.objects.push(object);
            util::create_stmt(P(util::create_method_call(
                str_to_ident("push"),
                P(util::create_self_field_expr(str_to_ident("objects"))),
                vec![P(util::create_var_expr(str_to_ident("object")))]
            ))),

            // let object = self.objects.last().unwrap();
            util::create_let_stmt(
                str_to_ident("object"),
                Some(P(util::create_method_call(
                    str_to_ident("unwrap"),
                    P(util::create_method_call(
                        str_to_ident("last"),
                        P(util::create_self_field_expr(str_to_ident("objects"))),
                        Vec::new()
                    )),
                    Vec::new()
                )))
            )
        ];

        for handler in self.handlers.iter() {
            stmts.push(util::create_stmt(P(handler.generate_add_check())));
        }

        util::impl_mut_method(
            str_to_ident("add"),
            vec![util::create_arg(
                str_to_ident("object"), 
                P(util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                ))
            )],
            Some(P(util::ty_from_ident(self.idx_name()))),
            P(util::create_block(
                stmts, 
                Some(P(util::create_call(
                    P(util::create_var_expr(self.idx_name())),
                    vec![P(util::create_var_expr(str_to_ident("idx")))]
                )))
            ))
        )
    }

    fn generate_fn_iter_impl(&self) -> ImplItem {
        util::impl_method(
            str_to_ident("iter"),
            Vec::new(),
            Some(P(util::path_param_ty(
                vec![str_to_ident("std"), str_to_ident("slice"), str_to_ident("Iter")],
                util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                )
            ))),
            P(util::create_block(
                Vec::new(),
                Some(P(util::create_method_call(
                    str_to_ident("iter"),
                    P(util::create_self_field_expr(str_to_ident("objects"))),
                    Vec::new()
                )))
            ))
        )
    }

    fn generate_fn_iter_mut_impl(&self) -> ImplItem {
        util::impl_mut_method(
            str_to_ident("iter_mut"),
            Vec::new(),
            Some(P(util::path_param_ty(
                vec![str_to_ident("std"), str_to_ident("slice"), str_to_ident("IterMut")],
                util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                )
            ))),
            P(util::create_block(
                Vec::new(),
                Some(P(util::create_method_call(
                    str_to_ident("iter_mut"),
                    P(util::create_self_field_expr(str_to_ident("objects"))),
                    Vec::new()
                )))
            ))
        )
    }

    fn generate_fn_remove_impl(&self) -> ImplItem {
        util::impl_mut_method(
            str_to_ident("remove"),
            vec![util::create_arg(
                str_to_ident("idx"),
                P(util::ty_from_ident(self.idx_name()))
            )],
            Some(P(util::param_ty_from_ident(
                str_to_ident("Option"),
                util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                )
            ))),
            P(util::create_block(
                Vec::new(),
                Some(P(util::create_method_call(
                    str_to_ident("and_then"),
                    P(util::create_method_call(
                        str_to_ident("cloned"),
                        P(util::create_method_call(
                            str_to_ident("get"),
                            P(util::create_self_field_expr(str_to_ident("idxs"))),
                            vec![P(util::create_tuple_field_expr(
                                P(util::create_var_expr(str_to_ident("idx"))),
                                0
                            ))],
                        )),
                        Vec::new()
                    )),
                    vec![P(util::create_closure_expr(
                        vec![util::create_arg(
                            str_to_ident("obj_idx"),
                            P(util::param_ty_from_ident(
                                str_to_ident("Option"),
                                util::ty_from_ident(str_to_ident("usize"))
                            ))
                        )],
                        P(util::create_block(
                            Vec::new(),
                            Some(P(util::create_method_call(
                                str_to_ident("map"),
                                P(util::create_var_expr(str_to_ident("obj_idx"))),
                                vec![P(util::create_closure_expr(
                                    vec![util::create_arg(
                                        str_to_ident("obj_idx"),
                                        P(util::ty_from_ident(str_to_ident("usize")))
                                    )],
                                    P(util::create_unsafe_block(
                                        vec![
                                            util::create_let_stmt(
                                                str_to_ident("obj"),
                                                Some(P(util::create_method_call(
                                                    str_to_ident("swap_remove"),
                                                    P(util::create_self_field_expr(str_to_ident("objects"))),
                                                    vec![P(util::create_var_expr(str_to_ident("obj_idx")))]
                                                )))
                                            ),
                                            util::create_stmt(P(util::create_assign_expr(
                                                P(util::create_deref_expr(P(util::create_method_call(
                                                    str_to_ident("unwrap"),
                                                    P(util::create_method_call(
                                                        str_to_ident("last_mut"),
                                                        P(util::create_self_field_expr(str_to_ident("idxs"))),
                                                        Vec::new()
                                                    )),
                                                    Vec::new()
                                                )))),
                                                P(util::create_call(
                                                    P(util::create_var_expr(str_to_ident("Some"))),
                                                    vec![P(util::create_var_expr(str_to_ident("obj_idx")))]
                                                ))
                                            ))),
                                            util::create_stmt(P(util::create_assign_expr(
                                                P(util::create_deref_expr(P(util::create_method_call(
                                                    str_to_ident("get_unchecked_mut"),
                                                    P(util::create_self_field_expr(str_to_ident("idxs"))),
                                                    vec![P(util::create_tuple_field_expr(
                                                        P(util::create_var_expr(str_to_ident("idx"))),
                                                        0
                                                    ))]
                                                )))),
                                                P(util::create_var_expr(str_to_ident("None")))
                                            )))
                                        ],
                                        Some(P(util::create_var_expr(str_to_ident("obj"))))
                                    ))
                                ))]
                            )))
                        ))
                    ))]
                )))
            ))
        )
    }

    fn generate_fn_get_impl(&self) -> ImplItem {
        util::impl_method(
            str_to_ident("get"),
            vec![util::create_arg(
                str_to_ident("idx"),
                P(util::ty_from_ident(self.idx_name()))
            )],
            Some(P(util::param_ty_from_ident(
                str_to_ident("Option"),
                util::ref_ty(P(util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                )))
            ))),
            P(util::create_block(
                Vec::new(),
                Some(P(util::create_method_call(
                    str_to_ident("and_then"),
                    P(util::create_method_call(
                        str_to_ident("cloned"),
                        P(util::create_method_call(
                            str_to_ident("get"),
                            P(util::create_self_field_expr(str_to_ident("idxs"))),
                            vec![P(util::create_tuple_field_expr(
                                P(util::create_var_expr(str_to_ident("idx"))),
                                0
                            ))],
                        )),
                        Vec::new()
                    )),
                    vec![P(util::create_closure_expr(
                        vec![util::create_arg(
                            str_to_ident("obj_idx"),
                            P(util::param_ty_from_ident(
                                str_to_ident("Option"),
                                util::ty_from_ident(str_to_ident("usize"))
                            ))
                        )],
                        P(util::create_block(
                            Vec::new(),
                            Some(P(util::create_method_call(
                                str_to_ident("map"),
                                P(util::create_var_expr(str_to_ident("obj_idx"))),
                                vec![P(util::create_closure_expr(
                                    vec![util::create_arg(
                                        str_to_ident("obj_idx"),
                                        P(util::ty_from_ident(str_to_ident("usize")))
                                    )],
                                    P(util::create_unsafe_block(
                                        Vec::new(),
                                        Some(P(util::create_method_call(
                                            str_to_ident("get_unchecked"),
                                            P(util::create_self_field_expr(str_to_ident("objects"))),
                                            vec![P(util::create_var_expr(str_to_ident("obj_idx")))]
                                        )))
                                    ))
                                ))]
                            )))
                        ))
                    ))]
                )))
            ))
        )
    }

    fn generate_fn_get_mut_impl(&self) -> ImplItem {
        util::impl_mut_method(
            str_to_ident("get_mut"),
            vec![util::create_arg(
                str_to_ident("idx"),
                P(util::ty_from_ident(self.idx_name()))
            )],
            Some(P(util::param_ty_from_ident(
                str_to_ident("Option"),
                util::mut_ref_ty(P(util::param_ty_from_ident(
                    str_to_ident("Box"),
                    util::ty_from_ident(self.object_name())
                )))
            ))),
            P(util::create_block(
                Vec::new(),
                Some(P(util::create_method_call(
                    str_to_ident("and_then"),
                    P(util::create_method_call(
                        str_to_ident("cloned"),
                        P(util::create_method_call(
                            str_to_ident("get"),
                            P(util::create_self_field_expr(str_to_ident("idxs"))),
                            vec![P(util::create_tuple_field_expr(
                                P(util::create_var_expr(str_to_ident("idx"))),
                                0
                            ))],
                        )),
                        Vec::new()
                    )),
                    vec![P(util::create_closure_expr(
                        vec![util::create_arg(
                            str_to_ident("obj_idx"),
                            P(util::param_ty_from_ident(
                                str_to_ident("Option"),
                                util::ty_from_ident(str_to_ident("usize"))
                            ))
                        )],
                        P(util::create_block(
                            Vec::new(),
                            Some(P(util::create_method_call(
                                str_to_ident("map"),
                                P(util::create_var_expr(str_to_ident("obj_idx"))),
                                vec![P(util::create_closure_expr(
                                    vec![util::create_arg(
                                        str_to_ident("obj_idx"),
                                        P(util::ty_from_ident(str_to_ident("usize")))
                                    )],
                                    P(util::create_unsafe_block(
                                        Vec::new(),
                                        Some(P(util::create_method_call(
                                            str_to_ident("get_unchecked_mut"),
                                            P(util::create_self_field_expr(str_to_ident("objects"))),
                                            vec![P(util::create_var_expr(str_to_ident("obj_idx")))]
                                        )))
                                    ))
                                ))]
                            )))
                        ))
                    ))]
                )))
            ))
        )
    }

    fn generate_impl(&self) -> Item {
        let mut fns = vec![
            self.generate_fn_new_impl(),
            self.generate_fn_add_impl(),
            self.generate_fn_iter_impl(),
            self.generate_fn_iter_mut_impl(),
            self.generate_fn_remove_impl(),
            self.generate_fn_get_impl(),
            self.generate_fn_get_mut_impl(),
        ];

        for handler in self.handlers.iter() {
            handler.generate_signal_impl(&mut fns);
        }

        util::create_impl(
            self.name,
            None,
            fns
        )
    }

    pub fn generate_object_impl(&self, thing: Ident, impls: &Vec<String>) -> Box<MacResult> {
        let mut items = Vec::new();

        for handler in self.handlers.iter() {
            items.extend_from_slice(&[
                util::impl_method_priv(
                    util::as_ident(handler.name),
                    Vec::new(),
                    Some(P(util::param_ty_from_ident(
                        str_to_ident("Option"),
                        util::ref_ty_from_ident(handler.name)
                    ))),
                    P(util::create_block(
                        Vec::new(),
                        Some(P(if impls.contains(&format!("{}", handler.name)) {
                            util::create_call(
                                P(util::create_var_expr(str_to_ident("Some"))),
                                vec![P(util::create_cast_expr(
                                        P(util::create_var_expr(str_to_ident("self"))),
                                        P(util::ref_ty_from_ident(handler.name))
                                ))]
                            )
                        } else {
                            util::create_var_expr(str_to_ident("None"))
                        })),
                    ))
                ),

                util::impl_mut_method_priv(
                    util::as_mut_ident(handler.name),
                    Vec::new(),
                    Some(P(util::param_ty_from_ident(
                        str_to_ident("Option"),
                        util::mut_ref_ty_from_ident(handler.name)
                    ))),
                    P(util::create_block(
                        Vec::new(),
                        Some(P(if impls.contains(&format!("{}", handler.name)) {
                            util::create_call(
                                P(util::create_var_expr(str_to_ident("Some"))),
                                vec![P(util::create_cast_expr(
                                        P(util::create_var_expr(str_to_ident("self"))),
                                        P(util::mut_ref_ty_from_ident(handler.name))
                                ))]
                            )
                        } else {
                            util::create_var_expr(str_to_ident("None"))
                        })),
                    ))
                )
            ]);
        }

        MacEager::items(SmallVector::one(P(util::create_impl(
            thing,
            Some(self.object_name()),
            items
        ))))
    }

    pub fn generate_ast(&self) -> Box<MacResult> {
        let mut items: Vec<P<Item>> = self.handlers.iter().map(|handler| P(handler.generate())).collect();
        items.extend_from_slice(&[
            P(self.generate_object_trait()),
            P(self.generate_idx_struct()),
            P(self.generate_struct()),
            P(self.generate_impl())
        ]);

        MacEager::items(SmallVector::many(items))
    }
}

impl HandlerInfo {
    pub fn new(name: Ident) -> HandlerInfo {
        HandlerInfo {
            name: name,
            reqs: Vec::new(),
            fns: Vec::new()
        }
    }
    
    pub fn add_requirement(&mut self, req: Ident) {
        self.reqs.push(req);
    }

    pub fn add_function(&mut self, function: HandlerFnInfo) {
        self.fns.push(function);
    }

    pub fn generate_as_self(&self) -> TraitItem {
        util::create_trait_method(
            util::as_ident(self.name),
            Vec::new(),
            Some(P(util::param_ty_from_ident(
                str_to_ident("Option"),
                util::ref_ty_from_ident(self.name)
            )))
        )
    }

    pub fn generate_as_self_mut(&self) -> TraitItem {
        util::create_mut_trait_method(
            util::as_mut_ident(self.name),
            Vec::new(),
            Some(P(util::param_ty_from_ident(
                str_to_ident("Option"),
                util::mut_ref_ty_from_ident(self.name)
            )))
        )
    }

    pub fn generate(&self) -> Item {
        util::create_trait(
            self.name,
            &self.reqs,
            &self.fns.iter().map(|function| function.generate()).collect()
        )
    }

    pub fn generate_signal_impl(&self, items: &mut Vec<ImplItem>) {
        for func in self.fns.iter() {
            let loop_block = util::create_block(
                vec![
                    // if i > len() { return }
                    util::create_stmt(P(util::create_if_expr(
                        P(util::create_binop_expr(
                            P(util::create_var_expr(str_to_ident("i"))),
                            BinOpKind::Ge,
                            P(util::create_method_call(
                                str_to_ident("len"),
                                P(util::create_self_field_expr(util::idxs_ident(self.name))),
                                Vec::new()
                            ))
                        )),
                        P(util::create_return_block(None)),
                        None
                    ))),

                    // let idx = *handler_idxs.get_unchecked(i);
                    util::create_let_stmt(
                        str_to_ident("idx"),
                        Some(P(util::create_deref_expr(P(util::create_method_call(
                            str_to_ident("get_unchecked"),
                            P(util::create_self_field_expr(util::idxs_ident(self.name))),
                            vec![P(util::create_var_expr(str_to_ident("i")))]
                        )))))
                    ),

                    util::create_let_stmt(
                        str_to_ident("idx"),
                        Some(P(util::create_deref_expr(P(util::create_method_call(
                            str_to_ident("get_unchecked"),
                            P(util::create_self_field_expr(str_to_ident("idxs"))),
                            vec![P(util::create_var_expr(str_to_ident("idx")))]
                        )))))
                    ),

                    util::create_stmt(P(util::create_if_let_expr(
                        P(util::create_tuple_struct_pat(
                            str_to_ident("Some"),
                            vec![str_to_ident("idx")]
                        )),
                        P(util::create_var_expr(str_to_ident("idx"))),
                        P(util::create_block(
                            vec![
                                util::create_stmt(P(util::create_method_call(
                                    func.dest_name,
                                    P(util::create_method_call(
                                        str_to_ident("unwrap"),
                                        P(util::create_method_call(
                                            util::as_mut_ident(self.name),
                                            P(util::create_method_call(
                                                str_to_ident("get_unchecked_mut"),
                                                P(util::create_self_field_expr(str_to_ident("objects"))),
                                                vec![P(util::create_var_expr(str_to_ident("idx")))]
                                            )),
                                            Vec::new()
                                        )),
                                        Vec::new(),
                                    )),
                                    func.args.iter().map(|arg| P(util::create_var_expr(arg.name))).collect()
                                ))),

                                util::create_stmt(P(util::create_assignop_expr(
                                    P(util::create_var_expr(str_to_ident("i"))),
                                    BinOpKind::Add,
                                    P(util::create_num_expr(1))
                                )))
                            ],
                            None
                        )),
                        Some(P(util::create_block_expr(P(util::create_block(
                            vec![util::create_stmt(P(util::create_method_call(
                                str_to_ident("swap_remove"),
                                P(util::create_self_field_expr(util::idxs_ident(self.name))),
                                vec![P(util::create_var_expr(str_to_ident("i")))]
                            )))],
                            None
                        ))))),
                    )))
                ],
                None
            );

            items.push(util::impl_mut_method(
                func.source_name,
                func.args.iter().map(|arg| arg.generate()).collect(),
                None,
                P(util::create_unsafe_block(
                    vec![
                        // let mut i = 0;
                        util::create_let_mut_stmt(
                            str_to_ident("i"),
                            Some(P(util::create_num_expr(0)))
                        ),

                        // loop { .. }
                        util::create_stmt(P(util::create_loop_expr(P(loop_block)))),
                    ],
                    None
                ))
            ));
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
                    util::create_stmt(P(util::create_method_call(
                        str_to_ident("push"),
                        P(util::create_self_field_expr(util::idxs_ident(self.name))),
                        vec![
                            P(util::create_var_expr(str_to_ident("idx")))
                        ]
                    ))),
                ],
                None
            )),

            None
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
            self.args.iter().map(|arg| arg.generate()).collect(),
            None
        )
    }
}

impl HandlerFnArg {
    pub fn new(name: Ident, ty: Ident, ptr: Option<Mutability>) -> HandlerFnArg {
        HandlerFnArg {
            name: name,
            ty: ty,
            ptr: ptr
        }
    }

    pub fn generate(&self) -> Arg {
        util::create_arg(self.name, match self.ptr {
            Some(Mutability::Immutable) => P(util::ref_ty_from_ident(self.ty)),
            Some(Mutability::Mutable) => P(util::mut_ref_ty_from_ident(self.ty)),
            None => P(util::ty_from_ident(self.ty))
        })
    }
}
