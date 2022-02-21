use crate::exec;

#[test]
fn strict_mode_global() {
    let scenario = r#"
        'use strict';
        let throws = false;
        try {
            delete Boolean.prototype;
        } catch (e) {
            throws = true;
        }
        throws
    "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn strict_mode_function() {
    let scenario = r#"
        let throws = false;
        function t() {
            'use strict';
            try {
                delete Boolean.prototype;
            } catch (e) {
                throws = true;
            }
        }
        t()
        throws
    "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn strict_mode_function_after() {
    let scenario = r#"
        function t() {
            'use strict';
        }
        t()
        let throws = false;
        try {
            delete Boolean.prototype;
        } catch (e) {
            throws = true;
        }
        throws
    "#;

    assert_eq!(&exec(scenario), "false");
}

#[test]
fn strict_mode_global_active_in_function() {
    let scenario = r#"
        'use strict'
        let throws = false;
        function a(){
            try {
                delete Boolean.prototype;
            } catch (e) {
                throws = true;
            }
        }
        a();
        throws
    "#;

    assert_eq!(&exec(scenario), "true");
}

#[test]
fn strict_mode_function_in_function() {
    let scenario = r#"
        let throws = false;
        function a(){
            try {
                delete Boolean.prototype;
            } catch (e) {
                throws = true;
            }
        }
        function b(){
            'use strict';
            a();
        }
        b();
        throws
    "#;

    assert_eq!(&exec(scenario), "false");
}

#[test]
fn strict_mode_function_return() {
    let scenario = r#"
        let throws = false;
        function a() {
            'use strict';
        
            return function () {
                try {
                    delete Boolean.prototype;
                } catch (e) {
                    throws = true;
                }
            }
        }
        a()();
        throws
    "#;

    assert_eq!(&exec(scenario), "true");
}
