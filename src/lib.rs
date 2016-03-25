//////////////////////////////////////////////////////////////////////////////
//  File: rust-handler/lib.rs
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

#![feature(plugin_registrar, rustc_private)]

#[macro_use]
extern crate syntax;

#[macro_use]
extern crate rustc;

#[macro_use]
extern crate rustc_plugin;

#[macro_use]
extern crate lazy_static;

use std::ops::Deref;
use std::sync::Mutex;
use std::collections::HashMap;

use rustc_plugin::Registry;

use syntax::parse::parser::Parser;
use syntax::ext::base::SyntaxExtension::IdentTT;
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult};
use syntax::codemap::Span;
use syntax::parse::token::{intern, Eof, Token};
use syntax::ast::*;

use system::*;

mod system;
mod util;

lazy_static! {
    pub static ref DEFINED_SYSTEMS: Mutex<HashMap<String, SystemInfo>> = Mutex::new(HashMap::new());
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("handlers_define_system"), IdentTT(Box::new(define_system_macro), None, false));

    reg.register_syntax_extension(intern("handlers_impl_object"), IdentTT(Box::new(impl_object_macro), None, false));
}

fn define_system_macro<'a>(ctx: &'a mut ExtCtxt, macro_span: Span, ident: Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'a> {
    let name = ident.name.as_str().deref().to_owned();

    let mut systems = DEFINED_SYSTEMS.lock().unwrap();
    if let Some(ref system) = systems.get(&name) {
        ctx.struct_span_err(macro_span, &format!("Redefinition of system '{}'", name))
            .span_note(system.span, "Previous definition was at:")
            .emit();

        return DummyResult::any(macro_span);
    }

    let mut system = SystemInfo::new(ident, macro_span);
    let mut parser = ctx.new_parser_from_tts(&tts);

    if parser.check(&Eof) {
        ctx.span_err(macro_span, "Expected list of handler definitions");
        return DummyResult::any(macro_span);
    }

    loop {
        match parse_handler_definition(ctx, &mut parser) {
            Some(handler) => system.add_handler(handler),
            None => break
        }

        if parser.check(&Eof) {
            break
        }
    }

    let result = system.generate_ast();
    systems.insert(name, system);
    result
}

fn impl_object_macro<'a>(ctx: &'a mut ExtCtxt, macro_span: Span, ident: Ident, tts: Vec<TokenTree>) -> Box<MacResult + 'a> {
    let name = ident.name.as_str().deref().to_owned();

    let systems = DEFINED_SYSTEMS.lock().unwrap();
    let system = if let Some(system) = systems.get(&name) {
        system
    } else {
        ctx.span_err(macro_span, &format!("Implementing object for undefined system '{}'", name));
        return DummyResult::any(macro_span);
    };

    let mut parser = ctx.new_parser_from_tts(&tts);

    let obj = match parser.parse_ident() {
        Ok(ident) => ident,

        Err(mut err) => {
            err.emit();
            return DummyResult::any(macro_span);
        }
    };

    if let Err(mut err) = parser.expect(&Token::Colon) {
        err.emit();
        return DummyResult::any(macro_span);
    }

    let mut impls = Vec::new();

    loop {
        if parser.check(&Eof) {
            break
        }

        match parser.parse_ident() {
            Ok(ident) => impls.push(format!("{}", ident)),

            Err(mut err) => {
                err.emit();
                return DummyResult::any(macro_span);
            }
        }

        if !parser.check(&Token::Comma) {
            break
        } else {
            parser.expect(&Token::Comma).unwrap();
        }
    }

    system.generate_object_impl(obj, &impls)
}

fn parse_handler_definition(ctx: &mut ExtCtxt, parser: &mut Parser) -> Option<HandlerInfo> {
    let mut handler = match parser.parse_ident() {
        Ok(ident) => HandlerInfo::new(ident),

        Err(mut err) => {
            err.emit();
            return None
        }
    };

    match parser.parse_token_tree() {
        Ok(TokenTree::Delimited(span, ref tts)) => {
            let mut handler_parser = ctx.new_parser_from_tts(&tts.tts);

            if handler_parser.check(&Eof) {
                ctx.span_err(span, "Expected delimited list of handler functions");
                return None
            }

            loop {
                if handler_parser.check(&Eof) {
                    break
                }

                match parse_handler_function_definition(ctx, &mut handler_parser) {
                    Some(function) => handler.add_function(function),
                    None => ()
                };

                if !handler_parser.check(&Token::Semi) {
                    break
                } else {
                    handler_parser.expect(&Token::Semi).unwrap();
                }
            }
        },

        Ok(ref tt) => {
            ctx.span_err(tt.get_span(), "Expected delimited list of handler functions");
            return None
        },

        Err(mut err) => {
            err.emit();
            return None
        }
    }

    Some(handler)
}

fn parse_handler_function_definition(ctx: &mut ExtCtxt, parser: &mut Parser) -> Option<HandlerFnInfo> {
    let source = match parser.parse_ident() {
        Ok(ident) => ident,

        Err(mut err) => {
            err.emit();
            return None
        }
    };

    let args = match parser.parse_token_tree() {
        Ok(TokenTree::Delimited(_, ref tts)) => {
            let mut arg_parser = ctx.new_parser_from_tts(&tts.tts);
            let mut args = Vec::new();

            loop {
                if arg_parser.check(&Eof) {
                    break
                }

                match parse_handler_function_arg(ctx, &mut arg_parser) {
                    Some(arg) => args.push(arg),
                    None => ()
                }

                if !arg_parser.check(&Token::Comma) {
                    break
                } else {
                    arg_parser.expect(&Token::Comma).unwrap();
                }
            }

            args
        },

        Ok(ref tt) => {
            ctx.span_err(tt.get_span(), "Expected function argument list");
            return None
        },

        Err(mut err) => {
            err.emit();
            return None
        }
    };

    if let Err(mut err) = parser.expect(&Token::FatArrow) {
        err.emit();
        return None
    };

    let dest = match parser.parse_ident() {
        Ok(ident) => ident,

        Err(mut err) => {
            err.emit();
            return None
        }
    };

    Some(HandlerFnInfo::new(source, dest, args))
}

fn parse_handler_function_arg(_: &mut ExtCtxt, parser: &mut Parser) -> Option<HandlerFnArg> {
    let name = match parser.parse_ident() {
        Ok(ident) => ident,

        Err(mut err) => {
            err.emit();
            return None
        }
    };

    if let Err(mut err) = parser.expect(&Token::Colon) {
        err.emit();
        return None
    }

    let ty = match parser.parse_ident() {
        Ok(ident) => ident,

        Err(mut err) => {
            err.emit();
            return None
        }
    };

    Some(HandlerFnArg::new(name, ty))
}
