use spin::lazy::Lazy;
use std::collections::HashSet;
use std::fmt::Debug;

use arbitrary::{size_hint, Arbitrary, Unstructured};
use boa_engine::syntax::ast::node::operator::assign::AssignTarget;
use boa_engine::syntax::ast::{
    node::{
        declaration::{BindingPatternTypeArray, BindingPatternTypeObject},
        iteration::IterableLoopInitializer,
        object::*,
        template::TemplateElement,
        *,
    },
    Const, Node,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[derive(Debug, PartialEq, Eq, Hash)]
struct Name {
    name: String,
}

/// Fuzz data which can be arbitrarily generated and used to test boa's parser, compiler, and vm
#[derive(Debug, Clone)]
pub struct FuzzData {
    source: String,
}

impl<'a> Arbitrary<'a> for FuzzData {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let first_name = Name::arbitrary(u)?; // need at least one
        let mut vars = HashSet::<Name>::arbitrary(u)?;
        vars.insert(first_name);
        let mut sample = StatementList::arbitrary(u)?;
        let mut interner = Interner::with_capacity(vars.len());
        let syms = vars
            .into_iter()
            .map(|var| interner.get_or_intern(var.name))
            .collect::<Vec<_>>();
        replace_syms(&syms, &mut sample);
        Ok(Self {
            source: sample.to_interned_string(&interner),
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        size_hint::and_all(&[
            Name::size_hint(depth),
            HashSet::<Name>::size_hint(depth),
            StatementList::size_hint(depth),
        ])
    }
}

impl FuzzData {
    /// Get the source represented by this fuzz data
    pub fn get_source(&self) -> &str {
        &self.source
    }
}

fn replace_syms(syms: &[Sym], sample: &mut StatementList) {
    replace_inner(syms, sample.items_mut().iter_mut().map(extendo).collect())
}

// there is certainly a much, much better way to do this
fn extendo<T>(node: &mut T) -> &'static mut T {
    unsafe { &mut *(node as *mut T) }
}

static ALPHA: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut all = Vec::new();
    all.extend(b'A'..b'Z');
    all.extend(b'a'..b'z');
    all
});

static ALPHANUM: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut all = Vec::new();
    all.extend(b'0'..b'9');
    all.extend(b'A'..b'Z');
    all.extend(b'a'..b'z');
    all
});

impl<'a> arbitrary::Arbitrary<'a> for Name {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let first = u8::arbitrary(u)?; // at least one
        let first = ALPHA[(first as usize) % ALPHA.len()];
        let mut chars: Vec<u8> = vec![first];
        let mut second: Vec<u8> = Arbitrary::arbitrary(u)?;
        second
            .iter_mut()
            .for_each(|c| *c = ALPHANUM[(*c as usize) % ALPHANUM.len()]);
        chars.extend(second);
        Ok(Self {
            name: String::from_utf8(chars).unwrap(),
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        size_hint::and(u8::size_hint(depth), Vec::<u8>::size_hint(depth))
    }
}

fn map_sym(syms: &[Sym], sym: &mut Sym) {
    *sym = syms[sym.as_raw().get() % syms.len()];
}

fn replace_block(nodes: &mut Vec<&'static mut Node>, block: &mut Block) {
    nodes.extend(block.items_mut().iter_mut().map(extendo));
}

fn replace_ident(syms: &[Sym], ident: &mut Identifier) {
    let mut sym = ident.sym();
    map_sym(syms, &mut sym);
    *ident = Identifier::new(sym);
}

fn replace_decl(syms: &[Sym], nodes: &mut Vec<&'static mut Node>, decl: &mut Declaration) {
    match decl {
        Declaration::Identifier { ident, init } => {
            replace_ident(syms, ident);
            if let Some(n) = init.as_mut() {
                nodes.push(extendo(n));
            }
        }
        Declaration::Pattern(declpattern) => replace_declpattern(syms, nodes, extendo(declpattern)),
    }
}

fn replace_decllist(
    syms: &[Sym],
    nodes: &mut Vec<&'static mut Node>,
    decllist: &mut DeclarationList,
) {
    match decllist {
        DeclarationList::Const(list) | DeclarationList::Let(list) | DeclarationList::Var(list) => {
            list.iter_mut()
                .for_each(|decl| replace_decl(syms, nodes, decl))
        }
    }
}

fn replace_declpattern(
    syms: &[Sym],
    nodes: &mut Vec<&'static mut Node>,
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
                            nodes.push(extendo(n));
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
                        stack.push(extendo(pattern));
                        if let Some(n) = default_init.as_mut() {
                            nodes.push(extendo(n));
                        }
                    }
                    BindingPatternTypeObject::RestGetConstField {
                        get_const_field,
                        excluded_keys,
                    } => {
                        nodes.push(extendo(get_const_field.obj_mut()));
                        map_sym(syms, get_const_field.field_mut());
                        excluded_keys.iter_mut().for_each(|s| map_sym(syms, s));
                    }
                });
                if let Some(n) = o.init_mut() {
                    nodes.push(extendo(n));
                }
            }
            DeclarationPattern::Array(a) => {
                a.bindings_mut().iter_mut().for_each(|bpta| match bpta {
                    BindingPatternTypeArray::Empty => {}
                    BindingPatternTypeArray::Elision => {}
                    BindingPatternTypeArray::SingleName {
                        ident,
                        default_init,
                    } => {
                        map_sym(syms, ident);
                        if let Some(n) = default_init.as_mut() {
                            nodes.push(extendo(n));
                        }
                    }
                    BindingPatternTypeArray::BindingPattern { pattern } => {
                        stack.push(extendo(pattern));
                    }
                    BindingPatternTypeArray::SingleNameRest { ident } => {
                        map_sym(syms, ident);
                    }
                    BindingPatternTypeArray::BindingPatternRest { pattern } => {
                        stack.push(extendo(pattern));
                    }
                    BindingPatternTypeArray::GetField { get_field }
                    | BindingPatternTypeArray::GetFieldRest { get_field } => {
                        nodes.push(extendo(get_field.obj_mut()));
                        nodes.push(extendo(get_field.field_mut()));
                    }
                    BindingPatternTypeArray::GetConstField { get_const_field } => {
                        nodes.push(extendo(get_const_field.obj_mut()));
                        map_sym(syms, get_const_field.field_mut());
                    }
                    BindingPatternTypeArray::GetConstFieldRest { get_const_field } => {
                        nodes.push(extendo(get_const_field.obj_mut()));
                        map_sym(syms, get_const_field.field_mut())
                    }
                });
                if let Some(n) = a.init_mut() {
                    nodes.push(extendo(n));
                }
            }
        }
    }
}

fn replace_afe(syms: &[Sym], nodes: &mut Vec<&'static mut Node>, afe: &mut AsyncFunctionExpr) {
    if let Some(sym) = afe.name_mut() {
        map_sym(syms, sym);
    }
    afe.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(afe.body_mut().iter_mut().map(extendo));
}

fn replace_age(syms: &[Sym], nodes: &mut Vec<&'static mut Node>, age: &mut AsyncGeneratorExpr) {
    if let Some(sym) = age.name_mut() {
        map_sym(syms, sym);
    }
    age.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(age.body_mut().iter_mut().map(extendo));
}

fn replace_fe(syms: &[Sym], nodes: &mut Vec<&'static mut Node>, fe: &mut FunctionExpr) {
    if let Some(sym) = fe.name_mut() {
        map_sym(syms, sym)
    }
    fe.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(fe.body_mut().items_mut().iter_mut().map(extendo));
}

fn replace_ge(syms: &[Sym], nodes: &mut Vec<&'static mut Node>, ge: &mut GeneratorExpr) {
    if let Some(sym) = ge.name_mut() {
        map_sym(syms, sym);
    }
    ge.parameters_mut()
        .items_mut()
        .iter_mut()
        .for_each(|fp| replace_fp(syms, nodes, fp));
    nodes.extend(ge.body_mut().items_mut().iter_mut().map(extendo));
}

fn replace_fp(syms: &[Sym], nodes: &mut Vec<&'static mut Node>, fp: &mut FormalParameter) {
    replace_decl(syms, nodes, fp.declaration_mut());
}

fn replace_ili(
    syms: &[Sym],
    nodes: &mut Vec<&'static mut Node>,
    ili: &mut IterableLoopInitializer,
) {
    match ili {
        IterableLoopInitializer::Identifier(i) => replace_ident(syms, i),
        IterableLoopInitializer::Var(d)
        | IterableLoopInitializer::Let(d)
        | IterableLoopInitializer::Const(d) => replace_decl(syms, nodes, d),
        IterableLoopInitializer::DeclarationPattern(declpattern) => {
            replace_declpattern(syms, nodes, declpattern)
        }
    }
}

fn replace_methdef(
    syms: &[Sym],
    nodes: &mut Vec<&'static mut Node>,
    methdef: &mut MethodDefinition,
) {
    match methdef {
        MethodDefinition::Get(fe) => replace_fe(syms, nodes, fe),
        MethodDefinition::Set(fe) => replace_fe(syms, nodes, fe),
        MethodDefinition::Ordinary(fe) => replace_fe(syms, nodes, fe),
        MethodDefinition::Generator(ge) => replace_ge(syms, nodes, ge),
        MethodDefinition::AsyncGenerator(age) => replace_age(syms, nodes, age),
        MethodDefinition::Async(afe) => replace_afe(syms, nodes, afe),
    }
}

fn replace_propdef(
    syms: &[Sym],
    nodes: &mut Vec<&'static mut Node>,
    propdef: &mut PropertyDefinition,
) {
    match propdef {
        PropertyDefinition::IdentifierReference(ir) => map_sym(syms, ir),
        PropertyDefinition::Property(pn, n) => {
            replace_propname(syms, nodes, pn);
            nodes.push(extendo(n));
        }
        PropertyDefinition::MethodDefinition(md, pn) => {
            replace_methdef(syms, nodes, md);
            replace_propname(syms, nodes, pn);
        }
        PropertyDefinition::SpreadObject(n) => {
            nodes.push(extendo(n));
        }
    }
}

fn replace_propname(syms: &[Sym], nodes: &mut Vec<&'static mut Node>, propname: &mut PropertyName) {
    match propname {
        PropertyName::Literal(l) => map_sym(syms, l),
        PropertyName::Computed(c) => nodes.push(extendo(c)),
    }
}

fn replace_inner(syms: &[Sym], mut nodes: Vec<&'static mut Node>) {
    while let Some(node) = nodes.pop() {
        match node {
            Node::ArrayDecl(orig) => nodes.extend(orig.as_mut().iter_mut().map(extendo)),
            Node::ArrowFunctionDecl(orig) => {
                if let Some(sym) = orig.name_mut() {
                    map_sym(syms, sym);
                }
                orig.params_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(orig.body_mut().items_mut().iter_mut().map(extendo));
            }
            Node::Assign(orig) => {
                match orig.lhs_mut() {
                    AssignTarget::Identifier(ident) => replace_ident(syms, ident),
                    AssignTarget::GetConstField(get_const_field) => {
                        nodes.push(extendo(get_const_field.obj_mut()));
                        map_sym(syms, get_const_field.field_mut());
                    }
                    AssignTarget::GetField(get_field) => {
                        nodes.push(extendo(get_field.obj_mut()));
                        nodes.push(extendo(get_field.field_mut()));
                    }
                    AssignTarget::DeclarationPattern(declpattern) => {
                        replace_declpattern(syms, &mut nodes, declpattern)
                    }
                }
                nodes.push(extendo(orig.rhs_mut()));
            }
            Node::AsyncFunctionDecl(orig) => {
                map_sym(syms, orig.name_mut());
                orig.parameters_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(orig.body_mut().items_mut().iter_mut().map(extendo));
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
                nodes.extend(orig.body_mut().iter_mut().map(extendo));
            }
            Node::AwaitExpr(orig) => nodes.push(orig.expr_mut()),
            Node::BinOp(orig) => {
                nodes.push(extendo(orig.lhs_mut()));
                nodes.push(extendo(orig.rhs_mut()));
            }
            Node::Block(orig) => replace_block(&mut nodes, orig),
            Node::Break(orig) => {
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::Call(orig) => {
                nodes.push(extendo(orig.expr_mut()));
                nodes.extend(orig.args_mut().iter_mut().map(extendo));
            }
            Node::ConditionalOp(orig) => {
                nodes.push(extendo(orig.cond_mut()));
                nodes.push(extendo(orig.if_true_mut()));
                nodes.push(extendo(orig.if_false_mut()));
            }
            Node::Const(Const::String(s)) => map_sym(syms, s),
            Node::Const(_) => {}
            Node::ConstDeclList(orig) => replace_decllist(syms, &mut nodes, orig),
            Node::Continue(orig) => {
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::DoWhileLoop(orig) => {
                nodes.push(extendo(orig.body_mut()));
                nodes.push(extendo(orig.cond_mut()));
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
                nodes.extend(orig.body_mut().items_mut().iter_mut().map(extendo));
            }
            Node::FunctionExpr(orig) => replace_fe(syms, &mut nodes, orig),
            Node::GetConstField(orig) => {
                nodes.push(extendo(orig.obj_mut()));
                map_sym(syms, orig.field_mut());
            }
            Node::GetField(orig) => {
                nodes.push(extendo(orig.obj_mut()));
                nodes.push(extendo(orig.field_mut()));
            }
            Node::ForLoop(orig) => {
                if let Some(n) = orig.init_mut() {
                    nodes.push(extendo(n));
                }
                if let Some(n) = orig.condition_mut() {
                    nodes.push(extendo(n));
                }
                if let Some(n) = orig.final_expr_mut() {
                    nodes.push(extendo(n));
                }
                if let Some(s) = orig.label_mut() {
                    map_sym(syms, s);
                }
                nodes.push(extendo(orig.body_mut()));
            }
            Node::ForInLoop(orig) => {
                replace_ili(syms, &mut nodes, orig.init_mut());
                nodes.push(extendo(orig.expr_mut()));
                nodes.push(extendo(orig.body_mut()));
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::ForOfLoop(orig) => {
                replace_ili(syms, &mut nodes, orig.init_mut());
                nodes.push(extendo(orig.iterable_mut()));
                nodes.push(extendo(orig.body_mut()));
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::If(orig) => {
                nodes.push(extendo(orig.cond_mut()));
                nodes.push(extendo(orig.body_mut()));
                if let Some(n) = orig.else_node_mut() {
                    nodes.push(extendo(n));
                }
            }
            Node::LetDeclList(orig) => replace_decllist(syms, &mut nodes, orig),
            Node::Identifier(orig) => replace_ident(syms, orig),
            Node::New(orig) => {
                nodes.push(extendo(orig.expr_mut()));
                nodes.extend(orig.args_mut().iter_mut().map(extendo));
            }
            Node::Object(orig) => {
                orig.properties_mut()
                    .iter_mut()
                    .for_each(|pd| replace_propdef(syms, &mut nodes, pd));
            }
            Node::Return(orig) => {
                if let Some(n) = orig.expr_mut() {
                    nodes.push(extendo(n));
                }
                if let Some(s) = orig.label_mut() {
                    map_sym(syms, s);
                }
            }
            Node::Switch(orig) => {
                nodes.push(extendo(orig.val_mut()));
                orig.cases_mut().iter_mut().for_each(|c| {
                    nodes.push(extendo(c.condition_mut()));
                    c.body_mut()
                        .items_mut()
                        .iter_mut()
                        .for_each(|n| nodes.push(extendo(n)));
                });
                if let Some(list) = orig.default_mut() {
                    nodes.extend(list.iter_mut().map(extendo));
                }
            }
            Node::Spread(orig) => nodes.push(orig.val_mut()),
            Node::TaggedTemplate(orig) => {
                nodes.push(extendo(orig.tag_mut()));
                orig.raws_mut().iter_mut().for_each(|s| map_sym(syms, s));
                orig.cookeds_mut()
                    .iter_mut()
                    .flat_map(|os| os.as_mut())
                    .for_each(|s| {
                        map_sym(syms, s);
                    });
                orig.exprs_mut()
                    .iter_mut()
                    .for_each(|n| nodes.push(extendo(n)));
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
                nodes.push(extendo(orig.target_mut()));
            }
            Node::VarDeclList(orig) => replace_decllist(syms, &mut nodes, orig),
            Node::WhileLoop(orig) => {
                nodes.push(extendo(orig.cond_mut()));
                nodes.push(extendo(orig.body_mut()));
                if let Some(sym) = orig.label_mut() {
                    map_sym(syms, sym);
                }
            }
            Node::Yield(orig) => {
                if let Some(n) = orig.expr_mut() {
                    nodes.push(extendo(n));
                }
            }
            Node::GeneratorDecl(orig) => {
                map_sym(syms, orig.name_mut());
                orig.parameters_mut()
                    .items_mut()
                    .iter_mut()
                    .for_each(|fp| replace_fp(syms, &mut nodes, fp));
                nodes.extend(orig.body_mut().iter_mut().map(extendo));
            }
            Node::GeneratorExpr(orig) => {
                replace_ge(syms, &mut nodes, orig);
            }
            Node::This | Node::Empty => {}
        }
    }
}
