Switch(Box<Node>, Box<[(Node, Box<[Node]>)]>, Option<Box<Node>>),



 Node::Switch(ref val_e, ref vals, ref default) => {
    let val = val_e.run(interpreter)?;
    let mut result = Value::null();
    let mut matched = false;
    for tup in vals.iter() {
        let cond = &tup.0;
        let block = &tup.1;
        if val.strict_equals(&cond.run(interpreter)?) {
            matched = true;
            let last_expr = block.last().expect("Block has no expressions");
            for expr in block.iter() {
                let e_result = expr.run(interpreter)?;
                if expr == last_expr {
                    result = e_result;
                }
            }
        }
    }
    if !matched {
        if let Some(default) = default {
            result = default.run(interpreter)?;
        }
    }
    Ok(result)
}