use gc_derive::{Finalize, Trace};
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, PartialEq, Debug, Finalize, Trace)]
/// Javascript import assignee
pub enum ImportAssignment {
    /// All exported values as local
    All(String),
    /// Default exported value
    Default(String),
    /// Named exported value
    NamedValuesBlock(Vec<(String, Option<String>)>),
}

impl Display for ImportAssignment {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            ImportAssignment::All(ref local) => write!(f, "* as {}", local),
            ImportAssignment::Default(ref local) => write!(f, "{}", local),
            ImportAssignment::NamedValuesBlock(ref values) => {
                let mut first = true;
                f.write_str("{ ")?;
                for i in values.iter() {
                    if !first {
                        f.write_str(", ")?;
                        first = false;
                    }
                    match i {
                        (value_name, Some(alias)) => {
                            f.write_fmt(format_args!("{} as {}", value_name, alias))?
                        }
                        (value_name, None) => f.write_fmt(format_args!("{}", value_name))?,
                    }
                }
                f.write_str(" }")?;
                Ok(())
            }
        }
    }
}
