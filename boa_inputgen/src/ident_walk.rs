//! Identifier and symbol walker for ensuring that generated inputs do not fail with "string
//! disappeared". The AST is walked by level-order traversal and Sym instances are replaced with
//! valid ones generated by arbitrary.

use boa_engine::syntax::ast::{
    node::{
        declaration::{BindingPatternTypeArray, BindingPatternTypeObject},
        iteration::IterableLoopInitializer,
        object::{MethodDefinition, PropertyDefinition, PropertyName},
        operator::assign::AssignTarget,
        template::TemplateElement,
        AsyncFunctionExpr, AsyncGeneratorExpr, Block, Declaration, DeclarationList,
        DeclarationPattern, FormalParameter, FunctionExpr, GeneratorExpr, Identifier,
        StatementList,
    },
    Const, Node,
};
use boa_interner::Sym;

/// Given this list of syms, walk the AST provided and replace any syms with a matching sym from
/// the list.
pub(crate) fn replace_syms(syms: &[Sym], sample: &mut StatementList) {
    replace_inner(
        syms,
        sample
            .items_mut()
            .iter_mut()
            .map(|n| unsafe { extend_lifetime(n) })
            .collect(),
    );
}

/// Extend the lifetime of an arbitrary reference temporarily. Only safe in this context because
/// we only hold these references during the walk, so they are *relatively* static. Holding
/// references to the AST while continuing to walk makes a double-mutable-reference scenario which
/// the borrow checker denies, but it's entirely safe since we only modify one Sym at a time and the
/// modification is idempotent. We don't need to pin because no data structures are modified, only
/// particular values.
unsafe fn extend_lifetime<'a, T>(node: &mut T) -> &'a mut T {
    &mut *(node as *mut T)
}

/// Primary mechanism by which symbols are replaced: we just change the value to one from the list
/// via modulo of the raw sym ID.
fn map_sym(syms: &[Sym], sym: &mut Sym) {
    *sym = syms[sym.as_raw().get() % syms.len()];
}

fn replace_block<'a>(nodes: &mut Vec<&'a mut Node>, block: &mut Block) {
    nodes.extend(
        block
            .items_mut()
            .iter_mut()
            .map(|n| unsafe { extend_lifetime(n) }),
    );
}

fn replace_ident(syms: &[Sym], ident: &mut Identifier) {
    let mut sym = ident.sym();
    map_sym(syms, &mut sym);
    *ident = Identifier::new(sym);
}

fn replace_decl<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, decl: &mut Declaration) {
    match decl {
        Declaration::Identifier { ident, init } => {
            replace_ident(syms, ident);
            if let Some(n) = init.as_mut() {
                nodes.push(unsafe { extend_lifetime(n) });
            }
        }
        Declaration::Pattern(declpattern) => {
            replace_declpattern(syms, nodes, unsafe { extend_lifetime(declpattern) })
        }
    }
}

fn replace_decllist<'a>(
    syms: &[Sym],
    nodes: &mut Vec<&'a mut Node>,
    decllist: &mut DeclarationList,
) {
    match decllist {
        DeclarationList::Const(list) | DeclarationList::Let(list) | DeclarationList::Var(list) => {
            list.iter_mut()
                .for_each(|decl| replace_decl(syms, nodes, decl));
        }
    }
}

fn replace_declpattern<'a>(
    syms: &[Sym],
    nodes: &mut Vec<&'a mut Node>,
    declpattern: &mut DeclarationPattern,
) {
    let mut stack = vec![declpattern];
    while let Some(declpattern) = stack.pop() {
        match declpattern {
            DeclarationPattern::Object(o) => {
                o.bindings_mut().iter_mut().for_each(|bpto| match bpto {
                    BindingPatternTypeObject::Empty => {}
                    BindingPatternTypeObject::SingleName {
                        ident,
                        property_name,
                        default_init,
                    } => {
                        map_sym(syms, ident);
                        map_sym(syms, property_name);
                        if let Some(n) = default_init.as_mut() {
                            nodes.push(unsafe { extend_lifetime(n) });
                        }
                    }
                    BindingPatternTypeObject::RestProperty {
                        ident,
                        excluded_keys,
                    } => {
                        map_sym(syms, ident);
                        excluded_keys.iter_mut().for_each(|sym| map_sym(syms, sym));
                    }
                    BindingPatternTypeObject::BindingPattern {
                        ident,
                        pattern,
                        default_init,
                    } => {
                        map_sym(syms, ident);
                        stack.push(unsafe { extend_lifetime(pattern) });
                        if let Some(n) = default_init.as_mut() {
                            nodes.push(unsafe { extend_lifetime(n) });
                        }
                    }
                    BindingPatternTypeObject::RestGetConstField {
                        get_const_field,
                        excluded_keys,
                    } => {
                        nodes.push(unsafe { extend_lifetime(get_const_field.obj_mut()) });
                        map_sym(syms, get_const_field.field_mut());
                        excluded_keys.iter_mut().for_each(|s| map_sym(syms, s));
                    }
                });
                if let Some(n) = o.init_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
            }
            DeclarationPattern::Array(a) => {
                a.bindings_mut().iter_mut().for_each(|bpta| match bpta {
                    BindingPatternTypeArray::Empty | BindingPatternTypeArray::Elision => {}
                    BindingPatternTypeArray::SingleName {
                        ident,
                        default_init,
                    } => {
                        map_sym(syms, ident);
                        if let Some(n) = default_init.as_mut() {
                            nodes.push(unsafe { extend_lifetime(n) });
                        }
                    }
                    BindingPatternTypeArray::BindingPattern { pattern }
                    | BindingPatternTypeArray::BindingPatternRest { pattern } => {
                        stack.push(unsafe { extend_lifetime(pattern) });
                    }
                    BindingPatternTypeArray::SingleNameRest { ident } => {
                        map_sym(syms, ident);
                    }
                    BindingPatternTypeArray::GetField { get_field }
                    | BindingPatternTypeArray::GetFieldRest { get_field } => {
                        nodes.push(unsafe { extend_lifetime(get_field.obj_mut()) });
                        nodes.push(unsafe { extend_lifetime(get_field.field_mut()) });
                    }
                    BindingPatternTypeArray::GetConstField { get_const_field } => {
                        nodes.push(unsafe { extend_lifetime(get_const_field.obj_mut()) });
                        map_sym(syms, get_const_field.field_mut());
                    }
                    BindingPatternTypeArray::GetConstFieldRest { get_const_field } => {
                        nodes.push(unsafe { extend_lifetime(get_const_field.obj_mut()) });
                        map_sym(syms, get_const_field.field_mut());
                    }
                });
                if let Some(n) = a.init_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
            }
        }
    }
}

fn replace_afe<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, afe: &mut AsyncFunctionExpr) {
    if let Some(sym) = afe.name_mut() {
        map_sym(syms, sym);
    }
    afe.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(
        afe.body_mut()
            .iter_mut()
            .map(|n| unsafe { extend_lifetime(n) }),
    );
}

fn replace_age<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, age: &mut AsyncGeneratorExpr) {
    if let Some(sym) = age.name_mut() {
        map_sym(syms, sym);
    }
    age.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(
        age.body_mut()
            .iter_mut()
            .map(|n| unsafe { extend_lifetime(n) }),
    );
}

fn replace_fe<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, fe: &mut FunctionExpr) {
    if let Some(sym) = fe.name_mut() {
        map_sym(syms, sym);
    }
    fe.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(
        fe.body_mut()
            .items_mut()
            .iter_mut()
            .map(|n| unsafe { extend_lifetime(n) }),
    );
}

fn replace_ge<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, ge: &mut GeneratorExpr) {
    if let Some(sym) = ge.name_mut() {
        map_sym(syms, sym);
    }
    ge.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(
        ge.body_mut()
            .items_mut()
            .iter_mut()
            .map(|n| unsafe { extend_lifetime(n) }),
    );
}

fn replace_fp<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, fp: &mut FormalParameter) {
    replace_decl(syms, nodes, fp.declaration_mut());
}

fn replace_ili<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, ili: &mut IterableLoopInitializer) {
    match ili {
        IterableLoopInitializer::Identifier(i) => replace_ident(syms, i),
        IterableLoopInitializer::Var(d)
        | IterableLoopInitializer::Let(d)
        | IterableLoopInitializer::Const(d) => replace_decl(syms, nodes, d),
        IterableLoopInitializer::DeclarationPattern(declpattern) => {
            replace_declpattern(syms, nodes, declpattern);
        }
    }
}

fn replace_methdef<'a>(
    syms: &[Sym],
    nodes: &mut Vec<&'a mut Node>,
    methdef: &mut MethodDefinition,
) {
    match methdef {
        MethodDefinition::Get(fe) | MethodDefinition::Set(fe) | MethodDefinition::Ordinary(fe) => {
            replace_fe(syms, nodes, fe)
        }
        MethodDefinition::Generator(ge) => replace_ge(syms, nodes, ge),
        MethodDefinition::AsyncGenerator(age) => replace_age(syms, nodes, age),
        MethodDefinition::Async(afe) => replace_afe(syms, nodes, afe),
    }
}

fn replace_propdef<'a>(
    syms: &[Sym],
    nodes: &mut Vec<&'a mut Node>,
    propdef: &mut PropertyDefinition,
) {
    match propdef {
        PropertyDefinition::IdentifierReference(ir) => map_sym(syms, ir),
        PropertyDefinition::Property(pn, n) => {
            replace_propname(syms, nodes, pn);
            nodes.push(unsafe { extend_lifetime(n) });
        }
        PropertyDefinition::MethodDefinition(md, pn) => {
            replace_methdef(syms, nodes, md);
            replace_propname(syms, nodes, pn);
        }
        PropertyDefinition::SpreadObject(n) => {
            nodes.push(unsafe { extend_lifetime(n) });
        }
    }
}

fn replace_propname<'a>(syms: &[Sym], nodes: &mut Vec<&'a mut Node>, propname: &mut PropertyName) {
    match propname {
        PropertyName::Literal(l) => map_sym(syms, l),
        PropertyName::Computed(c) => nodes.push(unsafe { extend_lifetime(c) }),
    }
}

/// Perform the AST walk. Method used here is a level-order traversal of the AST by using `nodes` as
/// a queue of nodes we still need to walk.
fn replace_inner<'a>(syms: &[Sym], mut nodes: Vec<&'a mut Node>) {
    while let Some(node) = nodes.pop() {
        match node {
            Node::ArrayDecl(orig) => nodes.extend(
                orig.as_mut()
                    .iter_mut()
                    .map(|n| unsafe { extend_lifetime(n) }),
            ),
            Node::ArrowFunctionDecl(orig) => {
                if let Some(sym) = orig.name_mut() {
                    map_sym(syms, sym);
                }
                orig.params_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(
                    orig.body_mut()
                        .items_mut()
                        .iter_mut()
                        .map(|n| unsafe { extend_lifetime(n) }),
                );
            }
            Node::Assign(orig) => {
                match orig.lhs_mut() {
                    AssignTarget::Identifier(ident) => replace_ident(syms, ident),
                    AssignTarget::GetConstField(get_const_field) => {
                        nodes.push(unsafe { extend_lifetime(get_const_field.obj_mut()) });
                        map_sym(syms, get_const_field.field_mut());
                    }
                    AssignTarget::GetField(get_field) => {
                        nodes.push(unsafe { extend_lifetime(get_field.obj_mut()) });
                        nodes.push(unsafe { extend_lifetime(get_field.field_mut()) });
                    }
                    AssignTarget::DeclarationPattern(declpattern) => {
                        replace_declpattern(syms, &mut nodes, declpattern);
                    }
                }
                nodes.push(unsafe { extend_lifetime(orig.rhs_mut()) });
            }
            Node::AsyncFunctionDecl(orig) => {
                map_sym(syms, orig.name_mut());
                orig.parameters_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(
                    orig.body_mut()
                        .items_mut()
                        .iter_mut()
                        .map(|n| unsafe { extend_lifetime(n) }),
                );
            }
            Node::AsyncFunctionExpr(orig) => {
                replace_afe(syms, &mut nodes, orig);
            }
            Node::AsyncGeneratorExpr(orig) => {
                replace_age(syms, &mut nodes, orig);
            }
            Node::AsyncGeneratorDecl(orig) => {
                map_sym(syms, orig.name_mut());
                orig.parameters_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(
                    orig.body_mut()
                        .iter_mut()
                        .map(|n| unsafe { extend_lifetime(n) }),
                );
            }
            Node::AwaitExpr(orig) => nodes.push(orig.expr_mut()),
            Node::BinOp(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.lhs_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.rhs_mut()) });
            }
            Node::Block(orig) => replace_block(&mut nodes, orig),
            Node::Break(orig) => {
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::Call(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.expr_mut()) });
                nodes.extend(
                    orig.args_mut()
                        .iter_mut()
                        .map(|n| unsafe { extend_lifetime(n) }),
                );
            }
            Node::ConditionalOp(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.cond_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.if_true_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.if_false_mut()) });
            }
            Node::Const(Const::String(s)) => map_sym(syms, s),
            Node::ConstDeclList(orig) | Node::LetDeclList(orig) | Node::VarDeclList(orig) => {
                replace_decllist(syms, &mut nodes, orig)
            }
            Node::Continue(orig) => {
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::DoWhileLoop(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.body_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.cond_mut()) });
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::FunctionDecl(orig) => {
                map_sym(syms, orig.name_mut());
                orig.parameters_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(
                    orig.body_mut()
                        .items_mut()
                        .iter_mut()
                        .map(|n| unsafe { extend_lifetime(n) }),
                );
            }
            Node::FunctionExpr(orig) => replace_fe(syms, &mut nodes, orig),
            Node::GetConstField(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.obj_mut()) });
                map_sym(syms, orig.field_mut());
            }
            Node::GetField(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.obj_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.field_mut()) });
            }
            Node::ForLoop(orig) => {
                if let Some(n) = orig.init_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
                if let Some(n) = orig.condition_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
                if let Some(n) = orig.final_expr_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
                if let Some(s) = orig.label_mut() {
                    map_sym(syms, s);
                }
                nodes.push(unsafe { extend_lifetime(orig.body_mut()) });
            }
            Node::ForInLoop(orig) => {
                replace_ili(syms, &mut nodes, orig.init_mut());
                nodes.push(unsafe { extend_lifetime(orig.expr_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.body_mut()) });
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::ForOfLoop(orig) => {
                replace_ili(syms, &mut nodes, orig.init_mut());
                nodes.push(unsafe { extend_lifetime(orig.iterable_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.body_mut()) });
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::If(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.cond_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.body_mut()) });
                if let Some(n) = orig.else_node_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
            }
            Node::Identifier(orig) => replace_ident(syms, orig),
            Node::New(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.expr_mut()) });
                nodes.extend(
                    orig.args_mut()
                        .iter_mut()
                        .map(|n| unsafe { extend_lifetime(n) }),
                );
            }
            Node::Object(orig) => {
                orig.properties_mut()
                    .iter_mut()
                    .for_each(|pd| replace_propdef(syms, &mut nodes, pd));
            }
            Node::Return(orig) => {
                if let Some(n) = orig.expr_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
                if let Some(s) = orig.label_mut() {
                    map_sym(syms, s);
                }
            }
            Node::Switch(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.val_mut()) });
                orig.cases_mut().iter_mut().for_each(|c| {
                    nodes.push(unsafe { extend_lifetime(c.condition_mut()) });
                    c.body_mut()
                        .items_mut()
                        .iter_mut()
                        .for_each(|n| nodes.push(unsafe { extend_lifetime(n) }));
                });
                if let Some(list) = orig.default_mut() {
                    nodes.extend(list.iter_mut().map(|n| unsafe { extend_lifetime(n) }));
                }
            }
            Node::Spread(orig) => nodes.push(orig.val_mut()),
            Node::TaggedTemplate(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.tag_mut()) });
                orig.raws_mut().iter_mut().for_each(|s| map_sym(syms, s));
                orig.cookeds_mut()
                    .iter_mut()
                    .flat_map(Option::as_mut)
                    .for_each(|s| {
                        map_sym(syms, s);
                    });
                orig.exprs_mut()
                    .iter_mut()
                    .for_each(|n| nodes.push(unsafe { extend_lifetime(n) }));
            }
            Node::TemplateLit(orig) => {
                orig.elements_mut().iter_mut().for_each(|te| match te {
                    TemplateElement::String(s) => map_sym(syms, s),
                    TemplateElement::Expr(n) => nodes.push(n),
                });
            }
            Node::Throw(orig) => nodes.push(orig.expr_mut()),
            Node::Try(orig) => {
                replace_block(&mut nodes, orig.block_mut());
                if let Some(c) = orig.catch_mut() {
                    if let Some(decl) = c.parameter_mut() {
                        replace_decl(syms, &mut nodes, decl);
                    }
                    replace_block(&mut nodes, c.block_mut());
                };
                if let Some(block) = orig.finally_mut() {
                    replace_block(&mut nodes, block);
                }
            }
            Node::UnaryOp(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.target_mut()) });
            }
            Node::WhileLoop(orig) => {
                nodes.push(unsafe { extend_lifetime(orig.cond_mut()) });
                nodes.push(unsafe { extend_lifetime(orig.body_mut()) });
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::Yield(orig) => {
                if let Some(n) = orig.expr_mut() {
                    nodes.push(unsafe { extend_lifetime(n) });
                }
            }
            Node::GeneratorDecl(orig) => {
                map_sym(syms, orig.name_mut());
                orig.parameters_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(
                    orig.body_mut()
                        .iter_mut()
                        .map(|n| unsafe { extend_lifetime(n) }),
                );
            }
            Node::GeneratorExpr(orig) => {
                replace_ge(syms, &mut nodes, orig);
            }
            Node::This | Node::Empty | Node::Const(_) => {}
        }
    }
}
