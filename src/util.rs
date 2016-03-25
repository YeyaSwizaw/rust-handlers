use syntax::ast::*;
use syntax::ptr::P;
use syntax::parse::token::str_to_ident;
use syntax::codemap::DUMMY_SP;

// https://github.com/rust-lang/rust/blob/master/src/librustc_lint/bad_style.rs#L148
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
