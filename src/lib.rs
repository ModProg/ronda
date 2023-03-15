use std::fmt::{self, Debug, Display, Formatter, Write};

use ron_edit::*;

#[must_use]
pub fn format(
    File {
        extentions,
        value,
        trailing_ws,
    }: &File,
) -> String {
    let mut out = String::new();
    let buf = &mut out;
    let c = Context {
        indent: Indent(0),
        nl: "\n",
    };

    extentions
        .iter()
        .try_for_each(|e| write!(buf, "{}", extention(e, c)))
        .expect("neither <String as fmt::Write> nor ron_edit's Display implementations error");
    if extentions.is_empty() {
        write!(buf, "{}", ws_lead_min(value, c, &crate::value))
    } else {
        write!(
            buf,
            "{}{}",
            ws_lead_nl(value, c, &crate::value),
            ws_min(trailing_ws, c)
        )
    }
    .expect("neither <String as fmt::Write> nor ron_edit's Display implementations error");
    write!(buf, "{}", ws_nl(trailing_ws, c))
        .expect("neither <String as fmt::Write> nor ron_edit's Display implementations error");
    out
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Indent(usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Context {
    indent: Indent,
    nl: &'static str,
}

impl Display for Indent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:1$}", "", self.0 * 4)
    }
}

impl Context {
    #[must_use]
    fn increase(mut self) -> Self {
        self.indent.0 += 1;
        self
    }
}

fn value<'a>(it: &'a Value, c @ Context { nl, .. }: Context) -> impl Display + 'a {
    Disp(move |f| match it {
        lit @ (Value::Int(_)
        | Value::Float(_)
        | Value::Bool(_)
        | Value::Unit(_)
        | Value::Char(_)) => {
            write!(f, "{lit}")
        }
        Value::Str(s) => write!(
            f,
            "{}",
            if nl == "\n" {
                format!("{s}").replace("\n\r", "\n")
            } else {
                debug_assert_eq!(nl, "\n\r");
                format!("{s}").replace("\n", "\n\r")
            }
        ),
        Value::List(List(list)) => write!(f, "[{}]", separated_split(list, &crate::value, c)),
        Value::Map(Map(fields)) => {
            write!(
                f,
                "{{{}}}",
                separated_split(
                    fields,
                    &|MapItem {
                          key,
                          after_key,
                          value,
                      },
                      c| Disp(move |f| write!(
                        f,
                        "{key}{}:{}",
                        ws_min(after_key, c),
                        ws_lead_single(value, c, &crate::value)
                    )),
                    c
                )
            )
        }
        Value::Tuple(Tuple { ident, fields }) => write!(
            f,
            "{}({})",
            option(ident, |ident| ws_followed_min(ident, c, &|s, _| s)),
            if fields.values.len() > 1 {
                separated_split(fields, &crate::value, c).to_string()
            } else {
                separated_min(fields, c, &crate::value).to_string()
            }
        ),
        Value::Struct(Struct { ident, fields }) => {
            write!(
                f,
                "{}({})",
                option(ident, |ident| ws_followed_min(ident, c, &|s, _| s)),
                separated_split(
                    fields,
                    &|NamedField {
                          key,
                          after_key,
                          value,
                      },
                      c| Disp(move |f| write!(
                        f,
                        "{key}{}:{}",
                        ws_min(after_key, c),
                        ws_lead_single(value, c, &crate::value),
                    )),
                    c
                )
            )
        }
    })
}

struct Disp<F: Fn(&mut Formatter<'_>) -> fmt::Result>(F);

impl<F: Fn(&mut Formatter<'_>) -> fmt::Result> Display for Disp<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

impl<F: Fn(&mut Formatter<'_>) -> fmt::Result> Debug for Disp<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", format!("{self}"))
    }
}

fn extention<'a>(
    WsLead {
        leading,
        content:
            Attribute {
                after_pound,
                after_exclamation,
                after_bracket,
                after_enable,
                extentions,
                after_paren,
            },
    }: &'a WsLead<Attribute>,
    c: Context,
) -> impl Display + 'a {
    Disp(move |f| {
        write!(
            f,
            "{}#{}!{}[{}enable{}({}){}]",
            ws_min(leading, c),
            ws_min(after_pound, c),
            ws_min(after_exclamation, c),
            ws_min(after_bracket, c),
            ws_min(after_enable, c),
            separated_min(extentions, c, &|&s, _| s),
            ws_min(after_paren, c),
        )
    })
}

fn without_trailing_nl(s: &str) -> &str {
    s.strip_suffix("\r\n").or(s.strip_suffix('\n')).unwrap_or(s)
}

fn start_with_space<'a>(s: &'a str) -> impl Display + 'a {
    Disp(move |f| {
        if s.starts_with(char::is_whitespace) {
            write!(f, "{s}")
        } else {
            write!(f, " {s}")
        }
    })
}

fn delimited_with_spaces<'a>(s: &'a str) -> impl Display + 'a {
    Disp(move |f| {
        match (
            s.starts_with(char::is_whitespace),
            s.ends_with(char::is_whitespace),
        ) {
            (true, true) => write!(f, "{s}"),
            (true, false) => write!(f, "{s} "),
            (false, true) => write!(f, " {s}"),
            (false, false) => write!(f, " {s} "),
        }
    })
}
#[test]
fn space_helpers() {
    assert_eq!(without_trailing_nl("hi\n"), "hi");
    assert_eq!(without_trailing_nl("hi\n\r\n"), "hi\n");
    assert_eq!(without_trailing_nl("hi"), "hi");
    assert_eq!(start_with_space("hi").to_string(), " hi");
    assert_eq!(start_with_space("\thi").to_string(), "\thi");
    assert_eq!(start_with_space(" hi").to_string(), " hi");
    assert_eq!(delimited_with_spaces("hi").to_string(), " hi ");
    assert_eq!(delimited_with_spaces("\thi").to_string(), "\thi ");
    assert_eq!(delimited_with_spaces(" hi").to_string(), " hi ");
    assert_eq!(delimited_with_spaces(" hi\t").to_string(), " hi\t");
}

fn ws_inner(
    f: &mut Formatter,
    ws: &Ws,
    Context { indent, nl }: Context,
    space: &mut &str,
) -> fmt::Result {
    match ws {
        Ws::LineComment(c) => {
            write!(
                f,
                "{space}//{}{nl}{indent}",
                start_with_space(without_trailing_nl(c))
            )?;
            *space = "";
        }
        Ws::Space(_) => {}
        Ws::BlockComment(c) => {
            write!(f, "{space}/*{}*/", delimited_with_spaces(c))?;
            *space = " ";
        }
    }
    Ok(())
}

fn ws_min<'a>(Whitespace(ws): &'a Whitespace, c: Context) -> impl Display + 'a {
    Disp(move |f| {
        let mut space = " ";
        ws.iter().try_for_each(|ws| ws_inner(f, ws, c, &mut space))
    })
}
fn ws_single<'a>(Whitespace(ws): &'a Whitespace, c: Context) -> impl Display + 'a {
    Disp(move |f| {
        let mut space = " ";
        ws.iter()
            .try_for_each(|ws| ws_inner(f, ws, c, &mut space))?;
        write!(f, "{space}")
    })
}
fn ws_nl<'a>(
    Whitespace(ws): &'a Whitespace,
    c @ Context { indent, nl }: Context,
) -> impl Display + 'a {
    Disp(move |f| {
        let mut space = " ";
        ws.iter()
            .try_for_each(|ws| ws_inner(f, ws, c, &mut space))?;
        if !space.is_empty() {
            write!(f, "{nl}{indent}")?;
        }
        Ok(())
    })
}

fn ws_wrapped_min<'a, T, D: Display>(
    WsWrapped {
        leading,
        content,
        following,
    }: &'a WsWrapped<T>,
    c: Context,
    format_content: &'a impl Fn(&'a T, Context) -> D,
) -> impl Display + 'a {
    Disp(move |f| {
        write!(
            f,
            "{}{}{}",
            ws_min(leading, c),
            format_content(content, c),
            ws_min(following, c)
        )
    })
}
fn ws_wrapped_leading_single<'a, T, D: Display>(
    WsWrapped {
        leading,
        content,
        following,
    }: &'a WsWrapped<T>,
    c: Context,
    format_content: &'a impl Fn(&'a T, Context) -> D,
) -> impl Display + 'a {
    Disp(move |f| {
        write!(
            f,
            "{}{}{}",
            ws_single(leading, c),
            format_content(content, c),
            ws_min(following, c)
        )
    })
}
fn ws_wrapped_leading_nl<'a, T, D: Display>(
    WsWrapped {
        leading,
        content,
        following,
    }: &'a WsWrapped<T>,
    c: Context,
    format_content: impl Fn(&'a T, Context) -> D + 'a,
) -> impl Display + 'a {
    Disp(move |f| {
        write!(
            f,
            "{}{}{}",
            ws_nl(leading, c),
            format_content(content, c),
            ws_min(following, c)
        )
    })
}

fn separated_min<'a, T, D: Display>(
    Separated {
        values,
        trailing_comma: _,
        trailing_ws,
    }: &'a Separated<'a, T>,
    c: Context,
    item: &'a impl Fn(&'a T, Context) -> D,
) -> impl Display + 'a {
    Disp(move |f| {
        if let Some((first, rest)) = values.split_first() {
            write!(f, "{}", ws_wrapped_min(first, c, item))?;
            if let Some((last, rest)) = rest.split_last() {
                write!(f, ",")?;
                rest.iter()
                    .try_for_each(|i| write!(f, "{},", ws_wrapped_leading_single(i, c, item)))?;
                write!(f, "{}", ws_wrapped_leading_single(last, c, item))?;
            }
            if !rest.is_empty() {}
        }
        write!(f, "{}", ws_min(trailing_ws, c))
    })
}
fn separated_split<'a, T, D: Display>(
    Separated {
        values,
        trailing_comma: _,
        trailing_ws,
    }: &'a Separated<'a, T>,
    item: &'a impl Fn(&'a T, Context) -> D,
    c: Context,
) -> impl Display + 'a {
    Disp(move |f| {
        values
            .iter()
            .try_for_each(|i| write!(f, "{},", ws_wrapped_leading_nl(i, c.increase(), item)))?;
        write!(f, "{}", ws_nl(trailing_ws, c))
    })
}

fn ws_lead_min<'a, T, D: Display>(
    WsLead { leading, content }: &'a WsLead<T>,
    c: Context,
    format_content: &'a impl Fn(&'a T, Context) -> D,
) -> impl Display + 'a {
    Disp(move |f| write!(f, "{}{}", ws_min(leading, c), format_content(content, c)))
}
fn ws_followed_min<'a, T, D: Display>(
    WsFollowed { content, following }: &'a WsFollowed<T>,
    c: Context,
    format_content: &'a impl Fn(&'a T, Context) -> D,
) -> impl Display + 'a {
    Disp(move |f| write!(f, "{}{}", format_content(content, c), ws_min(following, c)))
}
fn ws_lead_single<'a, T, D: Display>(
    WsLead { leading, content }: &'a WsLead<T>,
    c: Context,
    format_content: &'a impl Fn(&'a T, Context) -> D,
) -> impl Display + 'a {
    Disp(move |f| write!(f, "{}{}", ws_single(leading, c), format_content(content, c)))
}
fn ws_lead_nl<'a, T, D: Display>(
    WsLead { leading, content }: &'a WsLead<T>,
    c: Context,
    format_content: &'a impl Fn(&'a T, Context) -> D,
) -> impl Display + 'a {
    Disp(move |f| write!(f, "{}{}", ws_nl(leading, c), format_content(content, c)))
}
fn option<'a, T, D: Display>(
    opt: &'a Option<T>,
    format_content: impl Fn(&'a T) -> D + 'a,
) -> impl Display + 'a {
    Disp(move |f| {
        if let Some(value) = opt {
            write!(f, "{}", format_content(value))
        } else {
            Ok(())
        }
    })
}
