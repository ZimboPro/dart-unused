use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_till, take_until},
    character::complete::{alphanumeric1, multispace0},
    multi::many0,
    sequence::tuple,
};
use std::sync::OnceLock;

static INSTANCE: OnceLock<String> = OnceLock::new();

/// Set the class name to be used for localisation
///
/// NOTE: this needs to be set before calling `all_localisation`
pub fn set_class_name(class_name: &str) -> anyhow::Result<()> {
    INSTANCE
        .set(class_name.to_string())
        .expect("Failed to set class name");
    Ok(())
}

/// Parse all localisation keys from a string
pub fn all_localisation(input: &str) -> IResult<&str, Vec<&str>> {
    many0(localisation)(input)
}

/// Parse a single localisation key from a string
pub fn localisation(input: &str) -> IResult<&str, &str> {
    let (remaining, (_, _, _, _, _, _, _, _, key)) = tuple((
        take_until(INSTANCE.get().unwrap().as_str()),
        tag(INSTANCE.get().unwrap().as_str()),
        multispace0,
        tag("."),
        multispace0,
        alt((of_context, tag("current"), maybe_of)),
        multispace0,
        tag("."),
        is_alphanumeric_or_underscore,
    ))(input)?;
    Ok((remaining, key))
}

fn of_context(input: &str) -> IResult<&str, &str> {
    let (remaining, _s) = tuple((
        multispace0,
        tag("of("),
        alphanumeric1, // Generally 'context' but not guaranteed
        tag(")"),
    ))(input)?;
    Ok((remaining, ""))
}

fn maybe_of(input: &str) -> IResult<&str, &str> {
    let (remaining, _s) = tuple((
        multispace0,
        tag("maybeOf("),
        alphanumeric1, // Generally 'context' but not guaranteed
        tag(")"),
        multispace0,
        tag("?"),
    ))(input)?;
    Ok((remaining, ""))
}

pub(crate) fn is_alphanumeric_or_underscore(input: &str) -> IResult<&str, &str> {
    take_till(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')(input)
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use super::*;

    #[test]
    fn test_localisation() {
        Once::new().call_once(|| {
            let _ = INSTANCE.set("S".to_string());
        });
        let input = "S.of(context).app_name";
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = "S.current.app_name";
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = "S.maybeOf(context)?.app_name";
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn multi_line_test() {
        Once::new().call_once(|| {
            let _ = INSTANCE.set("S".to_string());
        });
        let input = r#"""S.of(context)
            .app_name"""#;
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#"""S.current
        .app_name"""#;
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#"""S.maybeOf(context)
        ?.app_name"""#;
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#"""S
        .of(context)
            .app_name"""#;
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#"""S
        .current
        .app_name"""#;
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#"""S
        .maybeOf(context)
        ?.app_name"""#;
        let expected = "app_name";
        let (_, actual) = localisation(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_multiple() {
        Once::new().call_once(|| {
            let _ = INSTANCE.set("S".to_string());
        });
        let input = r#""S.of(context).app_name
        S.of(context).app_name""#;
        let expected = vec!["app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#""S.current.app_name
        S.current.app_name""#;
        let expected = vec!["app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#""S.maybeOf(context)?.app_name
        S.maybeOf(context)?.app_name""#;
        let expected = vec!["app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);
        let input = r#""S.of(context).app_name, S.of(context)
        .app_name
        S.maybeOf(context)?.app_name""#;
        let expected = vec!["app_name", "app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_multiple_as_if_labels() {
        Once::new().call_once(|| {
            let _ = INSTANCE.set("S".to_string());
        });
        let input = r#""t: S.of(context).app_name,
        k: S.of(context).app_name""#;
        let expected = vec!["app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#""t: S.current.app_name,
        K:S.current.app_name""#;
        let expected = vec!["app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);

        let input = r#""t:S.maybeOf(context)?.app_name
        e:S.maybeOf(context)?.app_name""#;
        let expected = vec!["app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);
        let input = r#""d: S.of(context).app_name, k:S.of(context)
        .app_name
        s: S.maybeOf(context)?.app_name""#;
        let expected = vec!["app_name", "app_name", "app_name"];
        let (_, actual) = all_localisation(input).unwrap();
        assert_eq!(expected, actual);
    }
}
