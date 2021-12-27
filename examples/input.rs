use requestty::prompt::style::Stylize;
use requestty::Question;

fn map_err<E>(_: E) {}

fn parse_col(s: &str) -> Result<(u8, u8, u8), ()> {
    if !s.is_ascii() {
        return Err(());
    }

    let s = s.trim();

    if s.starts_with('#') {
        let (r, g, b, mult) = if s.len() == 4 {
            (1..2, 2..3, 3..4, 17)
        } else if s.len() == 7 {
            (1..3, 3..5, 5..7, 1)
        } else {
            return Err(());
        };

        Ok((
            mult * u8::from_str_radix(&s[r], 16).map_err(map_err)?,
            mult * u8::from_str_radix(&s[g], 16).map_err(map_err)?,
            mult * u8::from_str_radix(&s[b], 16).map_err(map_err)?,
        ))
    } else if s.starts_with("rgb(") && s.ends_with(')') {
        let mut s = &s[4..(s.len() - 1)];
        let r;
        let g;

        if let Some(pos) = s.find(',') {
            r = s[..pos].trim().parse().map_err(map_err)?;
            s = &s[(pos + 1)..];
        } else {
            return Err(());
        }

        if let Some(pos) = s.find(',') {
            g = s[..pos].trim().parse().map_err(map_err)?;
            s = &s[(pos + 1)..];
        } else {
            return Err(());
        }

        let b = s.trim().parse().map_err(map_err)?;

        Ok((r, g, b))
    } else {
        Err(())
    }
}

fn main() {
    let questions = vec![
        Question::input("first_name")
            .message("What's your first name")
            .build(),
        Question::input("last_name")
            .message("What's your last name")
            .default("Doe")
            .build(),
        Question::input("fav_color")
            .message("What's your favourite colour")
            .validate_on_key(|ans, _| parse_col(ans).is_ok())
            .validate(|ans, _| match parse_col(ans) {
                Ok(_) => Ok(()),
                Err(_) => Err("Please provide a valid css colour".into()),
            })
            .transform(|ans, _, backend| {
                let (r, g, b) = parse_col(ans).unwrap();

                backend.write_styled(&ans.rgb(r, g, b))?;
                writeln!(backend)
            })
            .build(),
    ];

    println!("{:#?}", requestty::prompt(questions));
}
