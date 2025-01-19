use boa_engine::{Context, JsResult, JsValue, JsVariant};
use crate::console::formatter;
use super::{Console, Logger};


struct TableChars {
    middle_middle: char,
    row_middle: char,
    left_middle: char,
    top_middle: char,
    right_middle: char,
    bottom_middle: char,

    top_right: char,
    top_left: char,
    bottom_right: char,
    bottom_left: char,

    left: &'static str,
    right: &'static str,
    middle: &'static str,
}

const table_chars: TableChars = TableChars {
    middle_middle: '─',
    row_middle: '┼',
    left_middle: '├',
    top_middle: '┬',
    right_middle: '┤',
    bottom_middle: '┴',
    top_right: '┐',
    top_left: '┌',
    bottom_right: '┘',
    bottom_left: '└',

    left: "│ ",
    right: " │",
    middle: " │ ",
};

pub(super) fn table_formatter(
    data: &[JsValue],
    context: &mut Context,
    console: &Console,
    logger: &impl Logger,
) -> JsResult<JsValue> {

    println!("Data: {:?}", data);


    fn render_row(value_vec: Vec<String>) -> String {
        return String::from("value");
    }

    fn print_table(value_vec: &Vec<String>) {
        println!("Printing table");
        let max_value_width = value_vec.iter().map(|value| value.len()).max().unwrap_or(0);
        let column_widths = value_vec
            .iter()
            .map(|value| value.len())
            .collect::<Vec<_>>();
        println!("Column widths: {:?}", column_widths);
        println!("Max value width: {:?}", max_value_width);
        println!("Vector length: {}", value_vec.len());

        let top_divider = column_widths
            .iter()
            .map(|width| table_chars.middle_middle.to_string().repeat(width + 2))
            .collect::<Vec<_>>();

        let top_left = table_chars.top_left.to_string();
        let top_middle = table_chars.top_middle.to_string();
        let top_right = table_chars.top_right.to_string();
        let left_middle = table_chars.left_middle.to_string();
        let row_middle = table_chars.row_middle.to_string();
        let right_middle = table_chars.right_middle.to_string();
        let bottom_left = table_chars.bottom_left.to_string();
        let bottom_middle = table_chars.bottom_middle.to_string();
        let bottom_right = table_chars.bottom_right.to_string();

        let mut result = format!(
            "{}{}{}\n",
            top_left,
            top_divider.join(&top_middle).to_string(),
            top_right
        );

        result.push_str(&format!("{}\n", render_row(value_vec.clone())));
        result.push_str(&format!(
            "{}{}{}\n",
            left_middle,
            top_divider.join(&row_middle).to_string(),
            right_middle
        ));
        result.push_str(&format!(
            "{}{}{}",
            bottom_left,
            top_divider.join(&bottom_middle).to_string(),
            bottom_right
        ));
        println!("{}", result);
    }

    let mut value_vec: Vec<String> = Vec::new();

    for arg in data {
        match arg.as_object() {
            // arg if let Some(obj) = arg.as_object()
            Some(arg) if arg.is_array() => {
                println!("This is array ------------");
                let array = arg.borrow();

                let key_value_array = array.properties().index_properties();
                for key_value in key_value_array {
                    match key_value.1.value().unwrap().variant() {
                        JsVariant::Integer32(integer) => {
                            value_vec.push(integer.to_string());
                        }
                        JsVariant::BigInt(bigint) => {
                            value_vec.push(bigint.to_string());
                        }
                        JsVariant::Boolean(boolean) => {
                            value_vec.push(boolean.to_string());
                        }
                        JsVariant::Float64(float64) => {
                            value_vec.push(float64.to_string());
                        }
                        JsVariant::String(string) => {
                            value_vec.push(string.to_std_string_escaped());
                        }
                        JsVariant::Symbol(symbol) => {
                            value_vec.push(symbol.to_string());
                        }
                        JsVariant::Null => {
                            value_vec.push(String::from("null"));
                        }
                        JsVariant::Undefined => {
                            value_vec.push(String::from("undefined"));
                        }
                        JsVariant::Object(obj) => {
                            // TODO: Implement object formatter. eg. {name: 'Xyz', age: 20} and so on.
                            value_vec.push(String::from("Object"));
                            // obj.borrow().properties()
                            // value_vec.push(JsObject::is_ordinary(&self) obj.prim)
                            // obj.fmt(format_args!("{:?}", obj));
                        }
                        _ => {
                            value_vec.push(String::from("undefined"));
                        }
                    }
                }
                print_table(&value_vec);
            }

            Some(arg) if arg.is_ordinary() => {
                println!("This is object ------------");
                let ordinary_object = arg.borrow();
                let key_value_array = ordinary_object.properties().index_properties();
                for key_value in key_value_array {
                    let key = key_value.0;
                    // let value = key_value.1.value().unwrap().to_string(context).unwrap();
                    println!("Obj: {key}");
                }
            }

            _ => {
                logger.log(formatter(data, context)?, &console.state, context);
            }
        }
    }

    Ok(JsValue::undefined())
}
