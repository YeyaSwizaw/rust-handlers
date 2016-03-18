use syntax::ast::*;
use syntax::ptr::P;
use syntax::parse::token::str_to_ident;
use syntax::codemap::DUMMY_SP;

pub fn indent_append(a: Ident, b: Ident) -> Ident {
    let str1 = format!("{}", a.name.as_str());
    let str2 = format!("{}", b.name.as_str());
    str_to_ident(&(str1 + &str2))
}

pub fn ty_from_ident(name: Ident) -> Ty {
    Ty {
        id: DUMMY_NODE_ID,
        span: DUMMY_SP,
        node: TyKind::Path(None, Path {
            span: DUMMY_SP,
            global: false,
            segments: vec![PathSegment {
                identifier: name,
                parameters: PathParameters::none()
            }]
        })
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

