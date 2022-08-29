use ui::events::{KeyCode, TestEvents};

mod helpers;

fn choices(len: usize) -> impl Iterator<Item = String> {
    (0..len).map(|choice| choice.to_string())
}

#[test]
fn test_validate() {
    let order_select = requestty::Question::order_select("name")
        .validate(|c, _| {
            if c[0].text() == "1" {
                Err("You have to make the bed first".to_string())
            } else {
                Ok(())
            }
        })
        .message("order select")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::Char(' ').into(),
        KeyCode::Down.into(),
        KeyCode::Char(' ').into(),
        KeyCode::Enter.into(),
    ]);

    let ans: Vec<_> = requestty::prompt_one_with(order_select, &mut backend, &mut events)
        .unwrap()
        .try_into_list_items()
        .unwrap()
        .into_iter()
        .map(|item| item.index)
        .collect();

    assert_eq!(ans, [3, 9]);
}

#[test]
fn test_filter() {
    let order_select = requestty::Question::order_select("name")
        .filter(|mut checked, _| {
            checked.rotate_left(1);
            checked
        })
        .message("multi select")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::Char(' ').into(),
        KeyCode::Down.into(),
        KeyCode::Char(' ').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(order_select, &mut backend, &mut events)
        .unwrap()
        .try_into_list_items()
        .unwrap()
        .into_iter()
        .map(|a| a.text)
        .collect::<Vec<_>>();

    // compute the expected answer
    let choices = choices(10).collect::<Vec<_>>();

    assert_eq!(ans, choices);
}

#[test]
fn test_transform() {
    let order_select = requestty::Question::order_select("name")
        .transform(|items, _, b| {
            b.set_fg(ui::style::Color::Magenta)?;
            for (i, item) in items.iter().enumerate() {
                write!(b, "{}: {}", item.index(), item.text())?;
                if i + 1 != items.len() {
                    write!(b, ", ")?;
                }
            }
            b.set_fg(ui::style::Color::Reset)
        })
        .message("multi select")
        .choices(choices(10));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(vec![
        KeyCode::Char(' ').into(),
        KeyCode::Down.into(),
        KeyCode::Char(' ').into(),
        KeyCode::Enter.into(),
    ]);

    let ans = requestty::prompt_one_with(order_select, &mut backend, &mut events)
        .unwrap()
        .try_into_list_items()
        .unwrap()
        .into_iter()
        .map(|a| a.text)
        .collect::<Vec<_>>();

    // compute the expected answer
    let choices = choices(10).collect::<Vec<_>>();

    assert_eq!(ans, choices);
}

#[test]
fn test_on_esc() {
    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_one_with(
        requestty::Question::order_select("name")
            .message("message")
            .choices(choices(10))
            .on_esc(requestty::OnEsc::Terminate),
        &mut backend,
        &mut events,
    );

    assert!(matches!(res, Err(requestty::ErrorKind::Aborted)));

    let size = (50, 20).into();
    let mut backend = helpers::SnapshotOnFlushBackend::new(size);
    let mut events = TestEvents::new(Some(KeyCode::Esc.into()));

    let res = requestty::prompt_with(
        Some(
            requestty::Question::order_select("name")
                .message("message")
                .choices(choices(10))
                .on_esc(requestty::OnEsc::SkipQuestion)
                .build(),
        ),
        &mut backend,
        &mut events,
    )
    .unwrap();

    assert!(res.is_empty());
}
