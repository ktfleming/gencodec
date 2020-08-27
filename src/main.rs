use heck::SnakeCase;
use indoc::formatdoc;
use regex::Regex;
use std::borrow::Cow;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    line = line.trim_end().to_owned();
    let case_class = parse(&line)?;
    println!("{}", case_class.companion_object());
    Ok(())
}

fn parse(input: &str) -> Result<CaseClass, Box<dyn Error>> {
    let input = input.replace("\n", "");
    let main_regex = Regex::new(
        r"(?x)
        case\ class\ 
        (?P<class>\w+)          # Class name
        (?:\[(?P<types>.+)\])?  # Type parameters
        \(                      # Literal paren to surround case class fields
        (?P<fields>.+)          # Field capture group
        \)                      # Literal paren to end case class
        ",
    )?;

    // Field name: just take everything before the colon
    let field_regex = Regex::new(r"^\s*(?P<field>\w+):")?;

    // Type param regex: everything up to the first non-word character, with optional variance
    // character
    let type_regex = Regex::new(r"^[+-]?(?P<type>\w+)")?;

    let captures = main_regex
        .captures(&input)
        .ok_or("Could not get capture groups for case class")?;

    let class_name = captures
        .name("class")
        .ok_or("Could not extract class name")?;
    let class_name = class_name.as_str().to_owned();

    let type_params: Result<Vec<String>, Box<dyn Error>> = captures
        .name("types")
        .map(|t| {
            t.as_str()
                .split(",")
                .map(|t| {
                    let type_name = type_regex
                        .captures(t)
                        .ok_or("Could not get capture groups for type params")?
                        .name("type")
                        .ok_or("Could not extract type param")?;
                    Ok(type_name.as_str().to_owned())
                })
                .collect()
        })
        .unwrap_or(Ok(vec![]));

    let fields = captures.name("fields").ok_or("Could not extract fields")?;

    let field_names: Result<Vec<String>, Box<dyn Error>> = fields
        .as_str()
        .split(",")
        .map(|f| {
            let field_name = field_regex
                .captures(f)
                .ok_or("Could not get capture groups for field name")?
                .name("field")
                .ok_or("Could not extract field name")?;
            Ok(field_name.as_str().to_owned())
        })
        .collect();

    Ok(CaseClass {
        name: class_name,
        type_params: type_params?,
        fields: field_names?,
    })
}

#[derive(Debug, Eq, PartialEq)]
struct CaseClass {
    name: String,
    type_params: Vec<String>, // [A] and such
    fields: Vec<String>,
}

impl CaseClass {
    fn is_generic(&self) -> bool {
        !self.type_params.is_empty()
    }

    fn companion_object(&self) -> String {
        // All the field names in snake case, joined into one comma-separated string
        let transformed_field_names = self
            .fields
            .iter()
            .map(|s| format!("\"{}\"", s.to_snake_case()))
            .collect::<Vec<String>>()
            .join(", ");

        // The tuple to use for the Encoder. `a` is the instance of the case class (could be any
        // arbitrary name).
        let tuple = format!(
            "a => ({})",
            self.fields
                .iter()
                .map(|s| format!("a.{}", s))
                .collect::<Vec<String>>()
                .join(", ")
        );

        let val_or_def = if self.is_generic() { "def" } else { "lazy val" };

        // Type params for the encoder/decoder functions, these should look like `A: Decoder`
        let function_type_params = |bound: &str| -> Cow<'static, str> {
            if self.is_generic() {
                let inside = self
                    .type_params
                    .iter()
                    .map(|t| format!("{}: {}", t, bound))
                    .collect::<Vec<String>>()
                    .join(", ");
                Cow::Owned(format!("[{}]", inside))
            } else {
                Cow::Borrowed("")
            }
        };

        // Just the type params. Like `[A, B]`. For use in the `apply` in the Decoder.
        let just_type_params = if self.is_generic() {
            Cow::Owned(format!(
                "[{}]",
                self.type_params
                    .iter()
                    .map(String::to_owned)
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        } else {
            Cow::Borrowed("")
        };

        // Class name with all type parameters. Like `Something[A, B]`
        let full_classname = if self.is_generic() {
            Cow::Owned(format!("{}{}", self.name, just_type_params))
        } else {
            Cow::Borrowed(&self.name)
        };

        formatdoc!(
            "object {short_classname} {{
              implicit {val_or_def} encoder{type_params_encoder}: Encoder[{full_classname}] = Encoder.forProduct{num}({fields})({tuple})

              implicit {val_or_def} decoder{type_params_decoder}: Decoder[{full_classname}] = Decoder.forProduct{num}({fields})({short_classname}.apply{just_type_params})
            }}",
            short_classname = self.name,
            full_classname = full_classname,
            val_or_def = val_or_def,
            type_params_encoder = function_type_params("Encoder"),
            type_params_decoder = function_type_params("Decoder"),
            num = self.fields.len(),
            fields = transformed_field_names,
            tuple = tuple,
            just_type_params = just_type_params
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse() {
        fn go(
            input: &str,
            expected_class_name: &str,
            expected_type_params: &[&str],
            expected_fields: &[&str],
        ) {
            assert_eq!(
                parse(input).unwrap(),
                CaseClass {
                    name: expected_class_name.to_owned(),
                    type_params: expected_type_params
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                    fields: expected_fields.into_iter().map(|s| s.to_string()).collect()
                }
            )
        }
        go("case class Person(age: Int)", "Person", &[], &["age"]);
        go(
            "case class Person(age: Int, favoriteFood: Food)",
            "Person",
            &[],
            &["age", "favoriteFood"],
        );
        go(
            "case class Person(
                age: Int,
                favoriteFood: Food
            )",
            "Person",
            &[],
            &["age", "favoriteFood"],
        );
        go(
            "case class Person(age: Int, favoriteFoods: List[Food])",
            "Person",
            &[],
            &["age", "favoriteFoods"],
        );
        go(
            "case class Generic[A](something: List[A])",
            "Generic",
            &["A"],
            &["something"],
        );
        go(
            "case class Generic[A <: B](something: List[A])",
            "Generic",
            &["A"],
            &["something"],
        );
        go(
            "case class Generic[A: Something](something: List[A])",
            "Generic",
            &["A"],
            &["something"],
        );
        go(
            "case class Generic[+A: Something](something: List[A])",
            "Generic",
            &["A"],
            &["something"],
        );
    }

    #[test]
    fn test_companion_object_basic() {
        let class = CaseClass {
            name: "Person".to_string(),
            type_params: vec![],
            fields: vec!["age".to_string(), "favoriteFood".to_string()],
        };
        assert_eq!(
            class.companion_object(),
            indoc!(
                r#"
                object Person {
                  implicit lazy val encoder: Encoder[Person] = Encoder.forProduct2("age", "favorite_food")(a => (a.age, a.favoriteFood))

                  implicit lazy val decoder: Decoder[Person] = Decoder.forProduct2("age", "favorite_food")(Person.apply)
                }"#
            )
        );
    }

    #[test]
    fn test_companion_object_generic() {
        let class = CaseClass {
            name: "Generic".to_string(),
            type_params: vec!["A".to_string()],
            fields: vec!["something".to_string()],
        };
        assert_eq!(
            class.companion_object(),
            indoc!(
                r#"
                object Generic {
                  implicit def encoder[A: Encoder]: Encoder[Generic[A]] = Encoder.forProduct1("something")(a => (a.something))

                  implicit def decoder[A: Decoder]: Decoder[Generic[A]] = Decoder.forProduct1("something")(Generic.apply[A])
                }"#
            )
        );
    }
}
