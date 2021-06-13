use crate::{
    backend::{TestBackend, TestBackendOp::*},
    events::KeyCode,
    style::Color,
    test_consts::*,
    widgets::Text,
    Widget,
};

use super::*;

const TERM_WIDTH: u16 = 100;

struct List<T> {
    vec: Vec<T>,
    selectable: Vec<bool>,
    page_size: usize,
    should_loop: bool,
}

impl<T> List<T> {
    fn new(vec: Vec<T>) -> Self {
        List {
            vec,
            selectable: Vec::new(),
            page_size: 15,
            should_loop: true,
        }
    }

    fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    fn with_should_loop(mut self, should_loop: bool) -> Self {
        self.should_loop = should_loop;
        self
    }

    fn with_selectable(mut self, selectable: Vec<bool>) -> Self {
        assert_eq!(selectable.len(), self.vec.len());
        self.selectable = selectable;
        self
    }
}

impl<T: Widget> super::List for List<T> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        _hovered: bool,
        mut layout: Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        self.vec[index].render(&mut layout, backend)
    }

    fn is_selectable(&self, index: usize) -> bool {
        *self.selectable.get(index).unwrap_or(&true)
    }

    fn page_size(&self) -> usize {
        self.page_size
    }

    fn should_loop(&self) -> bool {
        self.should_loop
    }

    fn height_at(&mut self, index: usize, mut layout: Layout) -> u16 {
        self.vec[index].height(&mut layout)
    }

    fn len(&self) -> usize {
        self.vec.len()
    }
}

/// Returns a Vec with things will render on a single line
fn single_line_vec(len: usize) -> Vec<String> {
    (0..len).map(|i| format!("{} list item", i)).collect()
}

/// Returns with first and last element taking 5 lines, and everything in between taking 2 lines
fn multi_line_list(len: usize) -> Vec<Text<String>> {
    std::iter::once(Text::new(LOREM.into()))
        .chain((1..(len - 1)).map(|i| {
            Text::new(format!(
                "{} {}",
                i,
                " list item".repeat(TERM_WIDTH as usize / 8)
            ))
        }))
        .chain(std::iter::once(Text::new(UNICODE.into())))
        .collect()
}

#[test]
fn test_height() {
    fn test(list: List<impl Widget>, height: u16, line_offset: u16) {
        let mut layout = Layout::new(line_offset, (100, 100).into());
        assert_eq!(Select::new(list).height(&mut layout), height);
    }

    test(List::new(single_line_vec(5)), 5, 0);
    test(List::new(single_line_vec(20)), 16, 10);

    test(List::new(multi_line_list(2)), 10, 0);
    test(List::new(multi_line_list(7)), 16, 10);
}

#[test]
fn test_selectable() {
    let list = List::new(single_line_vec(11)).with_selectable(vec![
        false, true, true, true, true, true, false, false, true, true, false,
    ]);

    let mut select = Select::new(list);
    select.update_heights(Layout::new(0, (100, 100).into()));
    select.init_page();

    assert_eq!(select.first_selectable, 1);
    assert_eq!(select.last_selectable, 9);

    assert_eq!(select.get_at(), 1);
    assert_eq!(select.prev_selectable(), 9);
    select.set_at(9);
    assert_eq!(select.next_selectable(), 1);
    select.set_at(1);

    select.set_at(7);
    assert_eq!(select.prev_selectable(), 5);
    select.set_at(5);
    assert_eq!(select.next_selectable(), 8);

    let list = select.finish().with_should_loop(false);

    let mut select = Select::new(list);
    select.update_heights(Layout::new(0, (100, 100).into()));
    select.init_page();

    assert_eq!(select.get_at(), 1);
    select.set_at(0);
    assert_eq!(select.prev_selectable(), 1);
    select.set_at(1);
    assert_eq!(select.prev_selectable(), 1);
    assert_eq!(select.next_selectable(), 2);

    select.set_at(7);
    assert_eq!(select.prev_selectable(), 5);
    select.set_at(5);
    assert_eq!(select.next_selectable(), 8);
    select.set_at(8);
    assert_eq!(select.next_selectable(), 9);
    select.set_at(9);
    assert_eq!(select.next_selectable(), 9);
    select.set_at(10);
    assert_eq!(select.next_selectable(), 9);
}

#[test]
fn test_update_heights() {
    let layout = Layout::new(0, (100, 100).into());

    let mut select = Select::new(List::new(single_line_vec(20)));
    select.update_heights(layout);
    let heights = &select.heights.as_ref().unwrap().heights[..];
    assert_eq!(heights.len(), 20);
    assert_eq!(select.height, 20);
    assert!(heights.iter().all(|&h| h == 1));

    let mut select = Select::new(List::new(multi_line_list(10)));
    select.update_heights(layout);
    let heights = &select.heights.as_ref().unwrap().heights[..];
    assert_eq!(heights.len(), 10);
    assert_eq!(select.height, 26);
    assert_eq!(heights[0], 5);
    assert_eq!(heights[9], 5);
    assert!(heights[1..9].iter().all(|&h| h == 2));
}

#[test]
fn test_at_outside_page() {
    let mut select = Select::new(List::new(single_line_vec(20)).with_page_size(10));
    select.update_heights(Layout::new(0, (100, 100).into()));
    select.init_page();

    select.at = 6;
    select.page_start = 5;
    select.page_end = 14;
    assert!(!select.at_outside_page());
    select.at = 10;
    assert!(!select.at_outside_page());
    select.at = 13;
    assert!(!select.at_outside_page());

    select.at = 2;
    assert!(select.at_outside_page());
    select.at = 5;
    assert!(select.at_outside_page());
    select.at = 14;
    assert!(select.at_outside_page());
    select.at = 18;
    assert!(select.at_outside_page());

    select.page_start = 15;
    select.page_end = 4;

    select.at = 1;
    assert!(!select.at_outside_page());
    select.at = 3;
    assert!(!select.at_outside_page());
    select.at = 16;
    assert!(!select.at_outside_page());
    select.at = 18;
    assert!(!select.at_outside_page());

    select.at = 4;
    assert!(select.at_outside_page());
    select.at = 9;
    assert!(select.at_outside_page());
    select.at = 15;
    assert!(select.at_outside_page());
}

#[test]
fn test_try_get_index() {
    let mut select = Select::new(List::new(single_line_vec(20)).with_page_size(10));
    select.update_heights(Layout::new(0, (100, 100).into()));
    select.init_page();

    select.at = 1;

    assert_eq!(select.try_get_index(-2), Some(19));
    assert_eq!(select.try_get_index(-1), Some(0));
    assert_eq!(select.try_get_index(1), Some(2));
    assert_eq!(select.try_get_index(2), Some(3));

    select.at = 18;

    assert_eq!(select.try_get_index(-2), Some(16));
    assert_eq!(select.try_get_index(-1), Some(17));
    assert_eq!(select.try_get_index(1), Some(19));
    assert_eq!(select.try_get_index(2), Some(0));

    select = Select::new(
        List::new(single_line_vec(20))
            .with_page_size(10)
            .with_should_loop(false),
    );

    select.at = 1;

    assert_eq!(select.try_get_index(-2), None);
    assert_eq!(select.try_get_index(-1), Some(0));
    assert_eq!(select.try_get_index(1), Some(2));
    assert_eq!(select.try_get_index(2), Some(3));

    select.at = 18;

    assert_eq!(select.try_get_index(-2), Some(16));
    assert_eq!(select.try_get_index(-1), Some(17));
    assert_eq!(select.try_get_index(1), Some(19));
    assert_eq!(select.try_get_index(2), None);
}

#[test]
fn test_adjust_page() {
    let mut select = Select::new(List::new(multi_line_list(10)).with_page_size(11));
    select.update_heights(Layout::new(0, (100, 100).into()));
    select.init_page();

    select.at = 1;

    select.adjust_page(Movement::Up);
    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 5);
    assert_eq!(select.page_end_height, 1);

    select.adjust_page(Movement::Down);
    assert_eq!(select.page_start, 9);
    assert_eq!(select.page_start_height, 2);
    assert_eq!(select.page_end, 2);
    assert_eq!(select.page_end_height, 1);

    select.at = 3;

    select.adjust_page(Movement::Down);
    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 3);
    assert_eq!(select.page_end, 4);
    assert_eq!(select.page_end_height, 1);

    select.at = 5;

    select.adjust_page(Movement::Up);
    assert_eq!(select.page_start, 4);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 9);
    assert_eq!(select.page_end_height, 1);

    select.adjust_page(Movement::Down);
    assert_eq!(select.page_start, 1);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 6);
    assert_eq!(select.page_end_height, 1);

    select.at = 8;

    select.adjust_page(Movement::Up);
    assert_eq!(select.page_start, 7);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 0);
    assert_eq!(select.page_end_height, 2);

    select.adjust_page(Movement::Down);
    assert_eq!(select.page_start, 4);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 9);
    assert_eq!(select.page_end_height, 1);

    let mut select = Select::new(
        List::new(multi_line_list(10))
            .with_page_size(11)
            .with_should_loop(false),
    );
    select.update_heights(Layout::new(0, (100, 100).into()));
    select.init_page();

    select.at = 0;

    select.adjust_page(Movement::Up);
    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 5);
    assert_eq!(select.page_end, 3);
    assert_eq!(select.page_end_height, 1);

    select.at = 3;

    select.adjust_page(Movement::Down);
    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 3);
    assert_eq!(select.page_end, 4);
    assert_eq!(select.page_end_height, 1);

    select.at = 5;

    select.adjust_page(Movement::Up);
    assert_eq!(select.page_start, 4);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 9);
    assert_eq!(select.page_end_height, 1);

    select.adjust_page(Movement::Down);
    assert_eq!(select.page_start, 1);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 6);
    assert_eq!(select.page_end_height, 1);

    select.at = 9;

    select.adjust_page(Movement::Down);
    assert_eq!(select.page_start, 6);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 9);
    assert_eq!(select.page_end_height, 5);
}

#[test]
fn test_init_page() {
    let layout = Layout::new(0, (100, 100).into());

    let mut select = Select::new(List::new(single_line_vec(10)));
    select.update_heights(layout);
    select.init_page();

    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 9);
    assert_eq!(select.page_end_height, 1);

    let mut select = Select::new(List::new(single_line_vec(20)));
    select.update_heights(layout);
    select.init_page();

    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 13);
    assert_eq!(select.page_end_height, 1);

    let mut select = Select::new(List::new(multi_line_list(4)));
    select.update_heights(layout);
    select.init_page();

    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 5);
    assert_eq!(select.page_end, 3);
    assert_eq!(select.page_end_height, 5);

    let mut select = Select::new(List::new(multi_line_list(5)));
    select.update_heights(layout);
    select.init_page();

    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 5);
    assert_eq!(select.page_end, 4);
    assert_eq!(select.page_end_height, 3);

    let mut select = Select::new(List::new(multi_line_list(10)));
    select.update_heights(layout);
    select.init_page();

    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 5);
    assert_eq!(select.page_end, 5);
    assert_eq!(select.page_end_height, 1);
}

#[test]
fn test_handle_key() {
    let layout = Layout::new(0, (100, 100).into());

    let mut select =
        Select::new(List::new(multi_line_list(10)).with_selectable(vec![
            false, true, true, true, false, true, false, true, true, true,
        ]));

    select.update_heights(layout);
    select.init_page();

    assert_eq!(select.get_at(), 1);

    assert!(select.handle_key(KeyCode::Up.into()));
    assert_eq!(select.get_at(), 9);
    assert_eq!(select.page_start, 8);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 2);
    assert_eq!(select.page_end_height, 1);

    assert!(select.handle_key(KeyCode::Down.into()));
    assert_eq!(select.get_at(), 1);
    assert_eq!(select.page_start, 8);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 2);
    assert_eq!(select.page_end_height, 1);

    assert!(select.handle_key(KeyCode::PageUp.into()));
    assert_eq!(select.get_at(), 8);
    assert_eq!(select.page_start, 7);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 1);
    assert_eq!(select.page_end_height, 1);

    assert!(select.handle_key(KeyCode::Home.into()));
    assert_eq!(select.get_at(), 1);
    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 7);
    assert_eq!(select.page_end_height, 1);

    assert!(!select.handle_key(KeyCode::Home.into()));

    assert!(select.handle_key(KeyCode::PageDown.into()));
    assert_eq!(select.get_at(), 7);
    assert_eq!(select.page_start, 1);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 8);
    assert_eq!(select.page_end_height, 1);

    assert!(select.handle_key(KeyCode::End.into()));
    assert_eq!(select.get_at(), 9);
    assert_eq!(select.page_start, 5);
    assert_eq!(select.page_start_height, 2);
    assert_eq!(select.page_end, 0);
    assert_eq!(select.page_end_height, 1);

    assert!(!select.handle_key(KeyCode::End.into()));

    let mut select = Select::new(
        List::new(multi_line_list(10))
            .with_selectable(vec![
                false, true, true, true, false, true, false, true, true, true,
            ])
            .with_should_loop(false),
    );
    select.update_heights(layout);
    select.init_page();

    assert!(!select.handle_key(KeyCode::Home.into()));
    assert!(!select.handle_key(KeyCode::Up.into()));
    assert!(!select.handle_key(KeyCode::PageUp.into()));
    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 5);
    assert_eq!(select.page_end, 5);
    assert_eq!(select.page_end_height, 1);

    select.at = 3;
    assert!(select.handle_key(KeyCode::PageUp.into()));
    assert_eq!(select.page_start, 0);
    assert_eq!(select.page_start_height, 5);
    assert_eq!(select.page_end, 5);
    assert_eq!(select.page_end_height, 1);

    assert!(select.handle_key(KeyCode::End.into()));
    assert_eq!(select.get_at(), 9);
    assert_eq!(select.page_start, 4);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 9);
    assert_eq!(select.page_end_height, 5);

    assert!(!select.handle_key(KeyCode::End.into()));
    assert!(!select.handle_key(KeyCode::Down.into()));
    assert!(!select.handle_key(KeyCode::PageDown.into()));

    select.at = 6;

    assert!(select.handle_key(KeyCode::PageDown.into()));
    assert_eq!(select.get_at(), 9);
    assert_eq!(select.page_start, 4);
    assert_eq!(select.page_start_height, 1);
    assert_eq!(select.page_end, 9);
    assert_eq!(select.page_end_height, 5);
}

#[test]
fn test_render() {
    // There are several duplications of the `SetCursor(_, _)` command in the following tests. This
    // is because `Text` happens to move to the correct position, but the select is not aware about
    // it, and sets the correct position as well.

    let size = (100, 100).into();
    let base_layout = Layout::new(0, size);
    let mut layout = base_layout;

    let mut ops = Vec::with_capacity(15);
    for (i, line) in single_line_vec(5).into_iter().enumerate() {
        ops.push(Write(line.into_bytes()));
        ops.push(SetCursor(0, i as u16 + 1));
        ops.push(SetCursor(0, i as u16 + 1));
    }

    Select::new(List::new(single_line_vec(5)))
        .render(&mut layout, &mut TestBackend::new(ops, size))
        .unwrap();
    assert_eq!(layout, base_layout.with_offset(0, 5));

    layout = base_layout.with_line_offset(10);

    let list = single_line_vec(20);
    let mut ops = Vec::with_capacity(32);
    ops.push(SetCursor(0, 1));
    for (i, line) in list[6..15].iter().enumerate() {
        ops.push(Write(line[..].into()));
        ops.push(SetCursor(0, i as u16 + 2));
        ops.push(SetCursor(0, i as u16 + 2));
    }
    ops.push(SetFg(Color::DarkGrey));
    ops.push(Write(b"(Move up and down to reveal more choices)".to_vec()));
    ops.push(SetFg(Color::Reset));
    ops.push(SetCursor(0, 11));

    let mut select = Select::new(List::new(list).with_page_size(10));
    select.update_heights(layout);
    select.init_page();
    select.set_at(13);

    select
        .render(&mut layout, &mut TestBackend::new(ops, size))
        .unwrap();

    assert_eq!(layout, base_layout.with_offset(0, 11));
    layout = base_layout;

    let list = single_line_vec(20);
    let mut ops = Vec::with_capacity(31);
    for (i, line) in list[16..].iter().chain(list[..5].iter()).enumerate() {
        ops.push(Write(line[..].into()));
        ops.push(SetCursor(0, i as u16 + 1));
        ops.push(SetCursor(0, i as u16 + 1));
    }
    ops.push(SetFg(Color::DarkGrey));
    ops.push(Write(b"(Move up and down to reveal more choices)".to_vec()));
    ops.push(SetFg(Color::Reset));
    ops.push(SetCursor(0, 10));

    let mut select = Select::new(List::new(list).with_page_size(10));
    select.update_heights(layout);
    select.init_page();
    select.at = 18;
    select.page_start = 16;
    select.page_end = 4;

    select
        .render(&mut layout, &mut TestBackend::new(ops, size))
        .unwrap();

    assert_eq!(layout, base_layout.with_offset(0, 10));

    let size = (120, 120).into();
    let base_layout = Layout::new(0, size).with_offset(20, 20);
    layout = base_layout.with_line_offset(10);

    let list = vec![
        Text::new(LOREM),
        Text::new("option 1 line 1\noption 1 line 2"),
        Text::new("option 2 line 1\noption 2 line 2"),
        Text::new("option 3 line 1\noption 3 line 2"),
        Text::new(UNICODE),
    ];

    let mut ops = Vec::with_capacity(28);
    ops.push(SetCursor(20, 21));
    ops.push(Write(b"imperdiet a, venenatis vitae, justo. Nullam dictum felis eu pede mollis pretium.".to_vec()));
    ops.push(SetCursor(20, 22));
    ops.push(SetCursor(20, 22));
    for i in 1..4 {
        ops.push(Write(format!("option {} line 1", i).into_bytes()));
        ops.push(SetCursor(20, 21 + i * 2));
        ops.push(Write(format!("option {} line 2", i).into_bytes()));
        ops.push(SetCursor(20, 22 + i * 2));
        ops.push(SetCursor(20, 22 + i * 2));
    }
    ops.push(Write("ǹɕǶǽũ ȥűǷŀȷÂǦǨÏǊ ýǡƎƭǃÁžƖţŝŬœĶ ɳƙŁŵŃŋŗ ǳÆŅɜŴô ħĲǗɧÝÙĝɸÿ ǝƬǄƫɌñÄç ɎƷɔȲƧ éďŅǒƿŅ üĲƪɮúǚĳǓɔÏǙǟ".into()));
    ops.push(SetCursor(20, 29));
    ops.push(Write("ǃóıÄ×ȤøŌɘŬÂ ȃŜʈǑƱļ ȶė÷ƝȣŞýş óɭǽƎȮ ŏŀƔȾřŞȩ ĚïƝƦʀƕĥǡǎÌʅ ĻɠȞīĈưĭÓĢÑ ǇĦƷűǐ¾đ ŊǂȘŰƒ ēɄɟɍƬč ɼ·ȄĶȸŦɉ ţĥŐŉŭ".into()));
    ops.push(SetCursor(20, 30));
    ops.push(SetCursor(20, 30));

    ops.push(SetFg(Color::DarkGrey));
    ops.push(Write(b"(Move up and down to reveal more choices)".to_vec()));
    ops.push(SetFg(Color::Reset));
    ops.push(SetCursor(20, 31));

    let mut select = Select::new(List::new(list).with_page_size(10));
    select.update_heights(layout);
    select.init_page();
    select.set_at(1);
    select.adjust_page(Movement::Up);

    select
        .render(&mut layout, &mut TestBackend::new(ops, size))
        .unwrap();

    assert_eq!(layout, base_layout.with_offset(20, 31));
}
