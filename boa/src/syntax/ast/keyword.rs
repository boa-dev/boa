use std::{
    error,
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};

/// Keywords are tokens that have special meaning in JavaScript.
///
/// In JavaScript you cannot use these reserved words as variables, labels, or function names.
///
/// More information:
///  - [ECMAScript reference](https://www.ecma-international.org/ecma-262/#sec-keywords)
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#Keywords
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Keyword {
    /// The `await` operator is used to wait for a [`Promise`][promise]. It can only be used inside an [async function][async-function].
    ///
    /// Syntax: `[rv] = await x;`
    ///
    /// The await expression causes async function execution to pause until a Promise is settled (that is, fulfilled or rejected),
    /// and to resume execution of the async function after fulfillment. When resumed, the value of the await expression is that of
    /// the fulfilled Promise.
    ///
    /// If the Promise is rejected, the await expression throws the rejected value.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-AwaitExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
    /// [promise]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
    /// [async-function]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
    Await,

    /// The break statement terminates the current loop, switch, or label statement and transfers program control to the statement following the terminated statement.
    ///
    /// Syntax: `break [label];`
    ///
    /// `label`(optional):
    ///  > Identifier associated with the label of the statement. If the statement is not a loop or switch, this is required.
    ///
    /// The break statement includes an optional label that allows the program to break out of a labeled statement.
    /// The break statement needs to be nested within the referenced label. The labeled statement can be any block statement;
    /// it does not have to be preceded by a loop statement.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-BreakStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
    Break,

    /// The `case` keyword is used to match against an expression in a switch statement.
    ///
    /// Syntax: `case x:`
    ///
    /// If the expression matches, the statements inside the case clause are executed until either the end of the
    /// switch statement or a break.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-CaseClause)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
    Case,

    /// The `catch` keyword is use in `catch`-block contains statements that specify what to do if an exception is thrown in the try-block.
    ///
    /// Syntax: `try { ... } catch(x) { ... }`
    ///
    /// `x`:
    ///  > An identifier to hold an exception object for the associated `catch`-block.
    ///
    /// If any statement within the try-block (or in a function called from within the try-block) throws an exception,
    /// control is immediately shifted to the catch-block. If no exception is thrown in the try-block, the catch-block is skipped.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-Catch)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Catch,

    /// The `class` keyword is used to creates a new class with a given name using prototype-based inheritance.
    ///
    /// Syntax: `class [name [extends otherName]] { /* `[`class body`][class-body]` */ }`
    ///
    /// `name`:
    ///  > The name of the class.
    ///
    /// `otherName`:
    ///  > The inherited class name.
    ///
    /// You can also define a class using a class expression. But unlike a class expression,
    /// a class declaration doesn't allow an existing class to be declared again and will throw a `SyntaxError` if attempted.
    ///
    /// The class body of a class declaration is executed in strict mode. The constructor method is optional.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ClassDeclaration)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
    /// [class-body]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes#Class_body_and_method_definitions
    Class,

    /// The continue statement terminates execution of the statements in the current iteration of the current or labeled loop,
    /// and continues execution of the loop with the next iteration.
    ///
    /// Syntax: `continue [label];`
    ///
    /// `label`(optional):
    ///  > Identifier associated with the label of the statement.
    ///
    /// The continue statement can include an optional label that allows the program to jump to the next iteration of a labeled
    /// loop statement instead of the current loop. In this case, the continue statement needs to be nested within this labeled statement.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ContinueStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
    Continue,

    /// Constants are block-scoped, much like variables defined using the let keyword.
    ///
    /// Syntax: `const name1 = value1 [, name2 = value2 [, ... [, nameN = valueN]]];`
    ///
    /// The const declaration creates a read-only reference to a value.
    /// It does not mean the value it holds is immutable—just that the variable identifier cannot be reassigned.
    ///
    /// This constant declaration whose scope can be either global or local to the block in which it is declared.
    /// Global constants do not become properties of the window object, unlike var variables.
    ///
    /// An initializer for a constant is required. You must specify its value in the same statement in which it's declared.
    /// (This makes sense, given that it can't be changed later.)
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-let-and-const-declarations)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
    Const,

    /// The `debugger` statement invokes any available debugging functionality, such as setting a breakpoint.
    ///
    /// Syntax: `debugger;`
    ///
    /// If no debugging functionality is available, this statement has no effect.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-debugger-statement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/debugger
    Debugger,

    /// The `default` keyword can be used  within a switch statement, or with an export statement.
    ///
    /// Syntax **in switch statement**:
    /// ```text
    /// switch (expression) {
    /// case value1:
    ///     //Statements executed when the result of expression matches value1
    ///     [break;]
    /// default:
    ///     //Statements executed when none of the values match the value of the expression
    ///     [break;]
    /// }
    /// ```
    ///
    /// The default keyword will help in any other case and executes the associated statement.
    ///
    /// Syntax **in export statement**: `export default name;`
    ///
    /// If you want to export a single value or need a fallback value for a module, a default export can be used.
    ///
    /// More information:
    ///  - [ECMAScript reference default clause](https://tc39.es/ecma262/#prod-DefaultClause)
    ///  - [ECMAScript reference default export](https://tc39.es/ecma262/#prod-ImportedDefaultBinding)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/default
    Default,

    /// The JavaScript `delete` operator removes a property from an object.
    ///
    /// Syntax: `delete x`
    ///
    /// Unlike what common belief suggests, the `delete` operator has **nothing** to do with directly freeing memory.
    /// Memory management is done indirectly via breaking references.
    ///
    /// The delete operator removes a given property from an object. On successful deletion,
    /// it will return `true`, else `false` will be returned.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-delete-operator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
    Delete,

    /// The `do` keyword is used in `do...while` statement creates a loop that executes a specified statement
    /// until the test condition evaluates to `false`.
    ///
    /// Syntax:
    /// ```javascript
    /// do
    ///    statement
    /// while (condition);
    /// ```
    ///
    /// `statement`
    ///  > A statement that is executed at least once and is re-executed each time the condition evaluates to `true`.
    ///
    /// `condition`
    ///  > An expression evaluated after each pass through the loop. If condition evaluates to true,
    /// the statement is re-executed. When condition evaluates to `false`, control passes to the statement following the `do...while`.
    ///
    /// The `do...while` loop iterates at least once and reiterates until the given condition return `false`.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-do-while-statement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
    Do,

    ///
    Else,

    /// The `enum` keyword
    ///
    /// Future reserved keywords.
    Enum,

    /// The export statement is used when creating JavaScript modules to export functions, objects, or primitive values from the module.
    ///
    /// Syntax:
    /// ```text
    /// // Exporting individual features
    /// export let name1, name2, …, nameN; // also var, const
    /// export let name1 = …, name2 = …, …, nameN; // also var, const
    /// export function functionName(){...}
    /// export class ClassName {...}
    ///
    /// // Export list
    /// export { name1, name2, …, nameN };
    ///
    /// // Renaming exports
    /// export { variable1 as name1, variable2 as name2, …, nameN };
    ///
    /// // Exporting destructured assignments with renaming
    /// export const { name1, name2: bar } = o;
    ///
    /// // Default exports
    /// export default expression;
    /// export default function (…) { … } // also class, function*
    /// export default function name1(…) { … } // also class, function*
    /// export { name1 as default, … };
    ///
    /// // Aggregating modules
    /// export * from …; // does not set the default export
    /// export * as name1 from …;
    /// export { name1, name2, …, nameN } from …;
    /// export { import1 as name1, import2 as name2, …, nameN } from …;
    /// export { default } from …;
    ///```
    ///
    /// There are two different types of export, named and default. You can have multiple named exports
    /// per module but only one default export.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-exports)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/export
    Export,

    /// The extends keyword is used in class declarations or class expressions to create a class which is a child of another class.
    ///
    /// Syntax: `class ChildClass extends ParentClass { ... }`
    ///
    /// The extends keyword can be used to subclass custom classes as well as built-in objects.
    ///
    /// The .prototype of the extension must be an Object or null.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ClassHeritage)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/extends
    Extends,

    /// The finally statement lets you execute code, after try and catch, regardless of the result.
    ///
    /// Syntax: `try { ... } [catch( ... ) { ... }] [finally { ... }]`
    ///
    /// The catch and finally statements are both optional, but you need to use one of them (if not both) while using the try statement.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-Finally)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
    Finally,

    /// The **`for` statement** creates a loop that consists of three optional expressions.
    ///
    /// Syntax:
    /// ```text
    /// for ([initialization]; [condition]; [final-expression])
    ///     statement
    ///
    /// ```
    /// `initialization`:
    ///  > An expression (including assignment expressions) or variable declaration evaluated once before the loop begins.
    ///
    /// `condition`:
    ///  > An expression to be evaluated before each loop iteration. If this expression evaluates to true, statement is executed.
    ///
    /// `final-expression`:
    ///  > An expression to be evaluated at the end of each loop iteration. This occurs before the next evaluation of condition.
    ///
    /// `statement`:
    ///  > A statement that is executed as long as the condition evaluates to true. To execute multiple statements within the loop,
    ///  > use a block statement (`{ ... }`) to group those statements.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-ForDeclaration)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
    For,

    /// The **`function` declaration** (function statement) defines a function with the specified parameters.
    ///
    /// Syntax:
    /// ```
    /// function name([param[, param,[..., param]]]) {
    ///     [statements]
    /// }
    /// ```
    ///
    /// `name`:
    ///  > The function name.
    ///
    /// `param`(optional):
    ///  > The name of an argument to be passed to the function. Maximum number of arguments varies in different engines.
    ///
    /// `statements`(optional):
    ///  > The statements which comprise the body of the function.
    ///
    /// A function created with a function declaration is a `Function` object and has all the properties, methods and behavior of `Function`.
    ///
    /// A function can also be created using an expression (see function expression).
    ///
    /// By default, functions return undefined. To return any other value, the function must have a return statement that specifies the value to return.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-terms-and-definitions-function)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
    Function,

    /// The **`if` statement** executes a statement if a specified condition is [`truthy`][truthy]. If the condition is [`falsy`][falsy], another statement can be executed.
    ///
    /// Syntax:
    /// ```
    /// if (condition)
    ///     statement1
    /// [else
    ///     statement2]
    /// ```
    ///
    /// `condition`:
    ///  > An [expression][expression] that is considered to be either [`truthy`][truthy] or [`falsy`][falsy].
    ///
    /// `statement1`:
    ///  > Statement that is executed if condition is truthy. Can be any statement, including further nested if statements.
    ///
    /// `statement2`:
    ///  > Statement that is executed if condition is [`falsy`][falsy] and the else clause exists. Can be any statement, including block statements and further nested if statements.
    ///
    /// Multiple `if...else` statements can be nested to create an else if clause.
    ///
    /// **Note** that there is no elseif (in one word) keyword in JavaScript.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-IfStatement)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
    /// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/truthy
    /// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/falsy
    /// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
    If,

    /// The **`in` operator** returns `true` if the specified property is in the specified object or its prototype chain.
    ///
    /// Syntax: `prop in object`
    ///
    /// `prop`:
    ///  > A string or symbol representing a property name or array index (non-symbols will be coerced to strings).
    ///
    /// `object`:
    ///  > Object to check if it (or its prototype chain) contains the property with specified name (`prop`).
    ///
    /// If you delete a property with the `delete` operator, the `in` operator returns `false` for that property.
    ///
    /// If you set a property to `undefined` but do not delete it, the `in` operator returns true for that property.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#prod-RelationalExpression)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/in
    In,

    /// The **`instanceof` operator** tests whether the `prototype` property of a constructor appears anywhere in the `prototype` chain of an object.
    ///
    /// Syntax: `object instanceof constructor`
    ///
    /// `object`:
    ///  > The object to test.
    ///
    /// `constructor`:
    ///  > Function to test against.
    ///
    /// The **`instanceof` operator** tests the presence of `constructor.prototype` in object's `prototype` chain.
    ///
    /// Note that the value of an `instanceof` test can change based on changes to the `prototype` property of constructors.
    /// It can also be changed by changing an object's prototype using `Object.setPrototypeOf`. It is also possible using the non-standard `__proto__` pseudo-property.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-instanceofoperator)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/instanceof
    InstanceOf,

    /// The static **`import` statement** is used to import bindings which are exported by another module.
    ///
    /// Syntax:
    /// ```
    /// import defaultExport from "module-name";
    /// import * as name from "module-name";
    /// import { export1 } from "module-name";
    /// import { export1 as alias1 } from "module-name";
    /// import { export1 , export2 } from "module-name";
    /// import { foo , bar } from "module-name/path/to/specific/un-exported/file";
    /// import { export1 , export2 as alias2 , [...] } from "module-name";
    /// import defaultExport, { export1 [ , [...] ] } from "module-name";
    /// import defaultExport, * as name from "module-name";
    /// import "module-name";
    /// var promise = import("module-name");
    /// ```
    ///
    /// `defaultExport`:
    ///  > Name that will refer to the default export from the module.
    ///
    /// `module-name`:
    ///  > The module to import from. This is often a relative or absolute path name to the .js file containing the module.
    ///  > Certain bundlers may permit or require the use of the extension; check your environment. Only single quoted and double quoted Strings are allowed.
    ///
    /// `name`:
    ///  > Name of the module object that will be used as a kind of namespace when referring to the imports.
    ///
    /// `exportN`:
    ///  > Name of the exports to be imported.
    ///
    /// `aliasN`:
    ///  > Names that will refer to the named imports.
    ///
    /// The `name` parameter is the name of the "module object" which will be used as a kind of namespace to refer to the exports.
    /// The `export` parameters specify individual named exports, while the `import * as name` syntax imports all of them.
    ///
    /// More information:
    ///  - [ECMAScript reference](https://tc39.es/ecma262/#sec-imports)
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import
    Import,

    /// The `let` keyword
    Let,
    /// The `new` keyword
    New,
    /// The `return` keyword
    Return,
    /// The `super` keyword
    Super,
    /// The `switch` keyword
    Switch,
    /// The `this` keyword
    This,
    /// The `throw` keyword
    Throw,
    /// The `try` keyword
    Try,
    /// The `typeof` keyword
    TypeOf,
    /// The `var` keyword
    Var,
    /// The `void` keyword
    Void,
    /// The `while` keyword
    While,
    /// The `with` keyword
    With,
    /// The 'yield' keyword
    Yield,
}

#[derive(Debug, Clone, Copy)]
pub struct KeywordError;
impl Display for KeywordError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "invalid token")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for KeywordError {
    fn description(&self) -> &str {
        "invalid token"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
impl FromStr for Keyword {
    type Err = KeywordError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "await" => Ok(Keyword::Await),
            "break" => Ok(Keyword::Break),
            "case" => Ok(Keyword::Case),
            "catch" => Ok(Keyword::Catch),
            "class" => Ok(Keyword::Class),
            "continue" => Ok(Keyword::Continue),
            "const" => Ok(Keyword::Const),
            "debugger" => Ok(Keyword::Debugger),
            "default" => Ok(Keyword::Default),
            "delete" => Ok(Keyword::Delete),
            "do" => Ok(Keyword::Do),
            "else" => Ok(Keyword::Else),
            "enum" => Ok(Keyword::Enum),
            "extends" => Ok(Keyword::Extends),
            "export" => Ok(Keyword::Export),
            "finally" => Ok(Keyword::Finally),
            "for" => Ok(Keyword::For),
            "function" => Ok(Keyword::Function),
            "if" => Ok(Keyword::If),
            "in" => Ok(Keyword::In),
            "instanceof" => Ok(Keyword::InstanceOf),
            "import" => Ok(Keyword::Import),
            "let" => Ok(Keyword::Let),
            "new" => Ok(Keyword::New),
            "return" => Ok(Keyword::Return),
            "super" => Ok(Keyword::Super),
            "switch" => Ok(Keyword::Switch),
            "this" => Ok(Keyword::This),
            "throw" => Ok(Keyword::Throw),
            "try" => Ok(Keyword::Try),
            "typeof" => Ok(Keyword::TypeOf),
            "var" => Ok(Keyword::Var),
            "void" => Ok(Keyword::Void),
            "while" => Ok(Keyword::While),
            "with" => Ok(Keyword::With),
            "yield" => Ok(Keyword::Yield),
            _ => Err(KeywordError),
        }
    }
}
impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match *self {
                Keyword::Await => "await",
                Keyword::Break => "break",
                Keyword::Case => "case",
                Keyword::Catch => "catch",
                Keyword::Class => "class",
                Keyword::Continue => "continue",
                Keyword::Const => "const",
                Keyword::Debugger => "debugger",
                Keyword::Default => "default",
                Keyword::Delete => "delete",
                Keyword::Do => "do",
                Keyword::Else => "else",
                Keyword::Enum => "enum",
                Keyword::Extends => "extends",
                Keyword::Export => "export",
                Keyword::Finally => "finally",
                Keyword::For => "for",
                Keyword::Function => "function",
                Keyword::If => "if",
                Keyword::In => "in",
                Keyword::InstanceOf => "instanceof",
                Keyword::Import => "import",
                Keyword::Let => "let",
                Keyword::New => "new",
                Keyword::Return => "return",
                Keyword::Super => "super",
                Keyword::Switch => "switch",
                Keyword::This => "this",
                Keyword::Throw => "throw",
                Keyword::Try => "try",
                Keyword::TypeOf => "typeof",
                Keyword::Var => "var",
                Keyword::Void => "void",
                Keyword::While => "while",
                Keyword::With => "with",
                Keyword::Yield => "yield",
            }
        )
    }
}
