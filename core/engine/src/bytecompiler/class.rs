use super::{ByteCompiler, Literal, Operand, ToJsString};
use crate::{
    js_string,
    vm::{BindingOpcode, CodeBlock, CodeBlockFlags, Opcode},
};
use boa_ast::{
    expression::Identifier,
    function::{
        ClassDeclaration, ClassElement, ClassElementName, ClassExpression, FormalParameterList,
        FunctionExpression,
    },
    property::{MethodDefinitionKind, PropertyName},
    scope::Scope,
    Expression,
};
use boa_gc::Gc;
use boa_interner::Sym;

// Static class elements that are initialized at a later time in the class creation.
enum StaticElement {
    // A static class block with it's function code.
    StaticBlock(Gc<CodeBlock>),

    // A static class field with it's function code, an optional name index and the information if the function is an anonymous function.
    StaticField((Gc<CodeBlock>, Option<u32>, bool)),
}

/// Describes the complete specification of a class.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ClassSpec<'a> {
    name: Option<Identifier>,
    super_ref: Option<&'a Expression>,
    constructor: Option<&'a FunctionExpression>,
    elements: &'a [ClassElement],
    has_binding_identifier: bool,
    name_scope: Option<&'a Scope>,
}

impl<'a> From<&'a ClassDeclaration> for ClassSpec<'a> {
    fn from(class: &'a ClassDeclaration) -> Self {
        Self {
            name: Some(class.name()),
            super_ref: class.super_ref(),
            constructor: class.constructor(),
            elements: class.elements(),
            has_binding_identifier: true,
            name_scope: Some(class.name_scope()),
        }
    }
}

impl<'a> From<&'a ClassExpression> for ClassSpec<'a> {
    fn from(class: &'a ClassExpression) -> Self {
        Self {
            name: class.name(),
            super_ref: class.super_ref(),
            constructor: class.constructor(),
            elements: class.elements(),
            has_binding_identifier: class.name().is_some(),
            name_scope: class.name_scope(),
        }
    }
}

impl ByteCompiler<'_> {
    /// This function compiles a class declaration or expression.
    ///
    /// The compilation of a class declaration and expression is mostly equal.
    /// A class declaration binds the resulting class object to it's identifier.
    /// A class expression leaves the resulting class object on the stack for following operations.
    pub(crate) fn compile_class(&mut self, class: ClassSpec<'_>, expression: bool) {
        // 11.2.2 Strict Mode Code - <https://tc39.es/ecma262/#sec-strict-mode-code>
        //  - All parts of a ClassDeclaration or a ClassExpression are strict mode code.
        let strict = self.strict();
        self.code_block_flags |= CodeBlockFlags::STRICT;

        let class_name = class
            .name
            .map_or(Sym::EMPTY_STRING, Identifier::sym)
            .to_js_string(self.interner());

        let outer_scope = self.push_declarative_scope(class.name_scope);

        let mut compiler = ByteCompiler::new(
            class_name.clone(),
            true,
            self.json_parse,
            self.variable_scope.clone(),
            self.lexical_scope.clone(),
            false,
            false,
            self.interner,
            self.in_with,
        );

        compiler.code_block_flags |= CodeBlockFlags::IS_CLASS_CONSTRUCTOR;

        if let Some(expr) = &class.constructor {
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
            let _ = compiler.push_scope(expr.scopes().function_scope());

            compiler.length = expr.parameters().length();
            compiler.params = expr.parameters().clone();
            compiler.parameter_scope = expr.scopes().parameter_scope();

            compiler.function_declaration_instantiation(
                expr.body(),
                expr.parameters(),
                false,
                true,
                false,
                expr.scopes(),
            );

            compiler.compile_statement_list(expr.body().statement_list(), false, false);

            compiler.emit_opcode(Opcode::PushUndefined);
        } else if class.super_ref.is_some() {
            // We push an empty, unused function scope since the compiler expects a function scope.
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
            let _ = compiler.push_scope(&Scope::new(compiler.lexical_scope.clone(), true));
            compiler.emit_opcode(Opcode::SuperCallDerived);
            compiler.emit_opcode(Opcode::BindThisValue);
        } else {
            // We push an empty, unused function scope since the compiler expects a function scope.
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
            let _ = compiler.push_scope(&Scope::new(compiler.lexical_scope.clone(), true));
            compiler.emit_opcode(Opcode::PushUndefined);
        }
        compiler.emit_opcode(Opcode::SetReturnValue);

        // 17. If ClassHeritageopt is present, set F.[[ConstructorKind]] to derived.
        compiler.code_block_flags.set(
            CodeBlockFlags::IS_DERIVED_CONSTRUCTOR,
            class.super_ref.is_some(),
        );

        let code = Gc::new(compiler.finish());
        let index = self.push_function_to_constants(code);
        self.emit_with_varying_operand(Opcode::GetFunction, index);

        self.emit_opcode(Opcode::Dup);
        if let Some(node) = &class.super_ref {
            self.compile_expr(node, true);
            self.emit_opcode(Opcode::PushClassPrototype);
        } else {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.emit_opcode(Opcode::SetClassPrototype);
        self.emit_opcode(Opcode::Swap);

        let count_label = self.emit_opcode_with_operand(Opcode::PushPrivateEnvironment);
        let mut count = 0;
        for element in class.elements {
            match element {
                ClassElement::MethodDefinition(m) => {
                    if let ClassElementName::PrivateName(name) = m.name() {
                        count += 1;
                        let index = self.get_or_insert_private_name(*name);
                        self.emit_u32(index);
                    }
                }
                ClassElement::PrivateFieldDefinition(field) => {
                    count += 1;
                    let index = self.get_or_insert_private_name(*field.name());
                    self.emit_u32(index);
                }
                ClassElement::PrivateStaticFieldDefinition(name, _) => {
                    count += 1;
                    let index = self.get_or_insert_private_name(*name);
                    self.emit_u32(index);
                }
                _ => {}
            }
        }
        self.patch_jump_with_target(count_label, count);

        let mut static_elements = Vec::new();
        let mut static_field_name_count = 0;

        if let Some(scope) = class.name_scope {
            let binding = scope.get_identifier_reference(class_name.clone());
            let index = self.get_or_insert_binding(binding);
            self.emit_opcode(Opcode::Dup);
            self.emit_binding_access(Opcode::PutLexicalValue, &index);
        }

        for element in class.elements {
            match element {
                ClassElement::MethodDefinition(m) => {
                    if !m.is_static() && !m.is_private() {
                        self.emit_opcode(Opcode::Swap);
                        self.emit_opcode(Opcode::Dup);
                    } else {
                        self.emit_opcode(Opcode::Dup);
                    }

                    match m.name() {
                        ClassElementName::PropertyName(PropertyName::Literal(name)) => {
                            self.method(m.into());
                            let index = self.get_or_insert_name((*name).into());
                            let opcode = match (m.is_static(), m.kind()) {
                                (true, MethodDefinitionKind::Get) => {
                                    Opcode::DefineClassStaticGetterByName
                                }
                                (true, MethodDefinitionKind::Set) => {
                                    Opcode::DefineClassStaticSetterByName
                                }
                                (true, _) => Opcode::DefineClassStaticMethodByName,
                                (false, MethodDefinitionKind::Get) => {
                                    Opcode::DefineClassGetterByName
                                }
                                (false, MethodDefinitionKind::Set) => {
                                    Opcode::DefineClassSetterByName
                                }
                                (false, _) => Opcode::DefineClassMethodByName,
                            };
                            self.emit_with_varying_operand(opcode, index);
                        }
                        ClassElementName::PropertyName(PropertyName::Computed(name)) => {
                            self.compile_expr(name, true);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.method(m.into());
                            let opcode = match (m.is_static(), m.kind()) {
                                (true, MethodDefinitionKind::Get) => {
                                    Opcode::DefineClassStaticGetterByValue
                                }
                                (true, MethodDefinitionKind::Set) => {
                                    Opcode::DefineClassStaticSetterByValue
                                }
                                (true, _) => Opcode::DefineClassStaticMethodByValue,
                                (false, MethodDefinitionKind::Get) => {
                                    Opcode::DefineClassGetterByValue
                                }
                                (false, MethodDefinitionKind::Set) => {
                                    Opcode::DefineClassSetterByValue
                                }
                                (false, _) => Opcode::DefineClassMethodByValue,
                            };
                            self.emit_opcode(opcode);
                        }
                        ClassElementName::PrivateName(name) => {
                            let index = self.get_or_insert_private_name(*name);
                            let opcode = match (m.is_static(), m.kind()) {
                                (true, MethodDefinitionKind::Get) => Opcode::SetPrivateGetter,
                                (true, MethodDefinitionKind::Set) => Opcode::SetPrivateSetter,
                                (true, _) => Opcode::SetPrivateMethod,
                                (false, MethodDefinitionKind::Get) => {
                                    Opcode::PushClassPrivateGetter
                                }
                                (false, MethodDefinitionKind::Set) => {
                                    Opcode::PushClassPrivateSetter
                                }
                                (false, _) => {
                                    self.emit(Opcode::RotateLeft, &[Operand::U8(3)]);
                                    self.emit_opcode(Opcode::Dup);
                                    self.emit(Opcode::RotateRight, &[Operand::U8(4)]);
                                    Opcode::PushClassPrivateMethod
                                }
                            };
                            self.method(m.into());
                            self.emit_with_varying_operand(opcode, index);
                        }
                    }

                    if !m.is_static() && !m.is_private() {
                        self.emit_opcode(Opcode::Swap);
                    }
                }
                ClassElement::FieldDefinition(field) => {
                    self.emit_opcode(Opcode::Dup);
                    match field.name() {
                        PropertyName::Literal(name) => {
                            self.emit_push_literal(Literal::String(
                                self.interner().resolve_expect(*name).into_common(false),
                            ));
                        }
                        PropertyName::Computed(name) => {
                            self.compile_expr(name, true);
                        }
                    }
                    let mut field_compiler = ByteCompiler::new(
                        js_string!(),
                        true,
                        self.json_parse,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                    );

                    // Function environment
                    field_compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = field_compiler.push_scope(field.scope());
                    let is_anonymous_function = if let Some(node) = &field.field() {
                        field_compiler.compile_expr(node, true);
                        node.is_anonymous_function_definition()
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                        false
                    };
                    field_compiler.emit_opcode(Opcode::SetReturnValue);

                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    let code = Gc::new(field_compiler.finish());
                    let index = self.push_function_to_constants(code);
                    self.emit_with_varying_operand(Opcode::GetFunction, index);
                    self.emit(
                        Opcode::PushClassField,
                        &[Operand::Bool(is_anonymous_function)],
                    );
                }
                ClassElement::PrivateFieldDefinition(field) => {
                    self.emit_opcode(Opcode::Dup);
                    let name_index = self.get_or_insert_private_name(*field.name());
                    let mut field_compiler = ByteCompiler::new(
                        class_name.clone(),
                        true,
                        self.json_parse,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                    );
                    field_compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = field_compiler.push_scope(field.scope());
                    if let Some(node) = field.field() {
                        field_compiler.compile_expr(node, true);
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                    }
                    field_compiler.emit_opcode(Opcode::SetReturnValue);

                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    let code = Gc::new(field_compiler.finish());
                    let index = self.push_function_to_constants(code);
                    self.emit_with_varying_operand(Opcode::GetFunction, index);
                    self.emit_with_varying_operand(Opcode::PushClassFieldPrivate, name_index);
                }
                ClassElement::StaticFieldDefinition(field) => {
                    let name_index = match field.name() {
                        PropertyName::Literal(name) => {
                            Some(self.get_or_insert_name((*name).into()))
                        }
                        PropertyName::Computed(name) => {
                            self.compile_expr(name, true);
                            self.emit(
                                Opcode::RotateRight,
                                &[Operand::U8(3 + static_field_name_count)],
                            );
                            static_field_name_count += 1;
                            None
                        }
                    };
                    let mut field_compiler = ByteCompiler::new(
                        class_name.clone(),
                        true,
                        self.json_parse,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                    );
                    field_compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = field_compiler.push_scope(field.scope());
                    let is_anonymous_function = if let Some(node) = &field.field() {
                        field_compiler.compile_expr(node, true);
                        node.is_anonymous_function_definition()
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                        false
                    };
                    field_compiler.emit_opcode(Opcode::SetReturnValue);

                    field_compiler.code_block_flags |= CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER;

                    let code = field_compiler.finish();
                    let code = Gc::new(code);

                    static_elements.push(StaticElement::StaticField((
                        code,
                        name_index,
                        is_anonymous_function,
                    )));
                }
                ClassElement::PrivateStaticFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    if let Some(node) = field {
                        self.compile_expr(node, true);
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    let index = self.get_or_insert_private_name(*name);
                    self.emit_with_varying_operand(Opcode::DefinePrivateField, index);
                }
                ClassElement::StaticBlock(block) => {
                    let mut compiler = ByteCompiler::new(
                        Sym::EMPTY_STRING.to_js_string(self.interner()),
                        true,
                        false,
                        self.variable_scope.clone(),
                        self.lexical_scope.clone(),
                        false,
                        false,
                        self.interner,
                        self.in_with,
                    );
                    compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
                    let _ = compiler.push_scope(block.scopes().function_scope());

                    compiler.function_declaration_instantiation(
                        block.statements(),
                        &FormalParameterList::default(),
                        false,
                        true,
                        false,
                        block.scopes(),
                    );

                    compiler.compile_statement_list(
                        block.statements().statement_list(),
                        false,
                        false,
                    );

                    let code = Gc::new(compiler.finish());
                    static_elements.push(StaticElement::StaticBlock(code));
                }
            }
        }

        for element in static_elements {
            match element {
                StaticElement::StaticBlock(code) => {
                    self.emit_opcode(Opcode::Dup);
                    let index = self.push_function_to_constants(code);
                    self.emit_with_varying_operand(Opcode::GetFunction, index);
                    self.emit_opcode(Opcode::SetHomeObject);
                    self.emit_with_varying_operand(Opcode::Call, 0);
                    self.emit_opcode(Opcode::Pop);
                }
                StaticElement::StaticField((code, name_index, is_anonymous_function)) => {
                    self.emit_opcode(Opcode::Dup);
                    self.emit_opcode(Opcode::Dup);
                    let index = self.push_function_to_constants(code);
                    self.emit_with_varying_operand(Opcode::GetFunction, index);
                    self.emit_opcode(Opcode::SetHomeObject);
                    self.emit_with_varying_operand(Opcode::Call, 0);
                    if let Some(name_index) = name_index {
                        self.emit_with_varying_operand(Opcode::DefineOwnPropertyByName, name_index);
                    } else {
                        self.emit(Opcode::RotateLeft, &[Operand::U8(5)]);
                        if is_anonymous_function {
                            self.emit_opcode(Opcode::Dup);
                            self.emit(Opcode::RotateLeft, &[Operand::U8(3)]);
                            self.emit_opcode(Opcode::Swap);
                            self.emit_opcode(Opcode::ToPropertyKey);
                            self.emit_opcode(Opcode::Swap);
                            self.emit(Opcode::SetFunctionName, &[Operand::U8(0)]);
                        } else {
                            self.emit_opcode(Opcode::Swap);
                        }
                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                    }
                }
            }
        }

        self.emit_opcode(Opcode::Swap);
        self.emit_opcode(Opcode::Pop);

        self.pop_declarative_scope(outer_scope);
        self.emit_opcode(Opcode::PopPrivateEnvironment);

        if !expression {
            self.emit_binding(BindingOpcode::InitVar, class_name);
        }

        // NOTE: Reset strict mode to before class declaration/expression evalutation.
        self.code_block_flags.set(CodeBlockFlags::STRICT, strict);
    }
}
