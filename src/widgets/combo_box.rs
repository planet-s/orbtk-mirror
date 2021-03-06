use orbclient::Renderer;
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use orbimage;
use orbclient;

use cell::{CheckSet, CloneCell};
use widgets::{Widget, VerticalPlacement, HorizontalPlacement};
use primitives::Image;
use draw::draw_box;
use event::Event;
use rect::Rect;
use point::Point;
use thickness::Thickness;
use theme::{Selector, Theme};
use traits::{Place, Style, Text};

static TOGGLE_ICON: &'static [u8; 703] = include_bytes!("../../res/icon-down-black.png");
static TOGGLE_ICON_ACTIVE: &'static [u8; 706] = include_bytes!("../../res/icon-down-white.png");

struct Entry {
    pub rect: Cell<Rect>,
    local_position: Cell<Point>,
    vertical_placement: Cell<VerticalPlacement>,
    horizontal_placement: Cell<HorizontalPlacement>,
    margin: Cell<Thickness>,
    children: RefCell<Vec<Arc<dyn Widget>>>,
    pub selector: CloneCell<Selector>,
    pub text: CloneCell<String>,
    pub text_offset: Cell<Point>,
    hover: Cell<bool>,
    pressed: Cell<bool>,
    index: u32,
    active: Cell<bool>,
}

impl Entry {
    fn new(text: &str, index: u32) -> Arc<Self> {
        Arc::new(Entry {
            rect: Cell::new(Rect::default()),
            local_position: Cell::new(Point::new(0, 0)),
            vertical_placement: Cell::new(VerticalPlacement::Absolute),
            horizontal_placement: Cell::new(HorizontalPlacement::Absolute),
            margin: Cell::new(Thickness::default()),
            children: RefCell::new(vec![]),
            selector: CloneCell::new(Selector::new(Some("combo-box-entry"))),
            text: CloneCell::new(String::from(text)),
            text_offset: Cell::new(Point::default()),
            hover: Cell::new(false),
            pressed: Cell::new(false),
            index,
            active: Cell::new(false),
        })
    }
}

impl Text for Entry {
    fn text<S: Into<String>>(&self, text: S) -> &Self {
        self.text.set(text.into());
        self
    }

    fn text_offset(&self, x: i32, y: i32) -> &Self {
        self.text_offset.set(Point::new(x, y));
        self
    }
}

impl Style for Entry {
    fn selector(&self) -> &CloneCell<Selector> {
        &self.selector
    }
}

impl Widget for Entry {
    fn rect(&self) -> &Cell<Rect> {
        &self.rect
    }

    fn local_position(&self) -> &Cell<Point> {
        &self.local_position
    }

    fn vertical_placement(&self) -> &Cell<VerticalPlacement> {
        &self.vertical_placement
    }

    fn horizontal_placement(&self) -> &Cell<HorizontalPlacement> {
        &self.horizontal_placement
    }

    fn margin(&self) -> &Cell<Thickness> {
        &self.margin
    }

    fn draw(&self, renderer: &mut dyn Renderer, _focused: bool, theme: &Theme) {
        let rect = self.rect.get();
        let offset = self.text_offset.get();
        let mut selector = self.selector.get();

        if self.hover.get() || self.active.get() {
            if self.active.get() {
                selector = selector.with_pseudo_class("active");
            } else {
                selector = selector.with_pseudo_class("hover");
            }

            draw_box(
                renderer,
                Rect::new(rect.x, rect.y, rect.width, rect.height),
                theme,
                &selector,
            );
        }

        let mut point = Point::new(rect.x + offset.x, rect.y + rect.height as i32 / 2 - 8);
        for c in self.text.get().chars() {
            if point.x + 8 <= rect.x +  rect.width as i32 - offset.x {
                renderer.char(point.x, point.y, c, theme.color("color", &selector));
            }
            point.x += 8;
        }
    }

    fn event(&self, event: Event, _focused: bool, redraw: &mut bool, caught: &mut bool) -> bool {
        match event {
            Event::Mouse { point, left_button, .. } => {
                let mut click = false;

                let rect = self.rect.get();
                if rect.contains(point) {
                    if self.hover.check_set(true) {
                        *redraw = true;
                    }

                    if left_button {
                        if self.pressed.check_set(true) {
                            *redraw = true;
                        }
                    } else {
                        if self.pressed.check_set(false) {
                            click = true;
                            self.hover.set(false);
                            *redraw = true;
                        }
                    }

                    *caught = true;
                } else {
                    if self.hover.check_set(false) {
                        *redraw = true;
                    }

                    if !left_button {
                        if self.pressed.check_set(false) {
                            *redraw = true;
                        }
                    }
                }

                if click {
                    return true;
                }
            }
            _ => (),
        }

        false
    }

    fn name(&self) -> &str {
        "ComboBoxEntry"
    }

    fn children(&self) -> &RefCell<Vec<Arc<dyn Widget>>> {
        &self.children
    }
}

pub struct ComboBox {
    pub rect: Cell<Rect>,
    children: RefCell<Vec<Arc<dyn Widget>>>,
    local_position: Cell<Point>,
    vertical_placement: Cell<VerticalPlacement>,
    horizontal_placement: Cell<HorizontalPlacement>,
    margin: Cell<Thickness>,
    pub selector: CloneCell<Selector>,
    pressed: Cell<bool>,
    activated: Cell<bool>,
    pub offset: Cell<Point>,
    selected: Cell<Option<u32>>,
    entries: RefCell<Vec<Arc<Entry>>>,
    text: CloneCell<String>,
    flyout_height: Cell<u32>,
    toggle_icon: RefCell<Option<Arc<Image>>>,
    toggle_icon_active: RefCell<Option<Arc<Image>>>,
}

impl ComboBox {
    pub fn new() -> Arc<ComboBox> {
        let toggle_icon = RefCell::new(None);
        if let Ok(icon) = orbimage::parse_png(TOGGLE_ICON) {
            *toggle_icon.borrow_mut() = Some(Image::from_image(icon))
        };

        let toggle_icon_active = RefCell::new(None);
        if let Ok(icon) = orbimage::parse_png(TOGGLE_ICON_ACTIVE) {
            *toggle_icon_active.borrow_mut() = Some(Image::from_image(icon))
        };

        Arc::new(ComboBox {
            rect: Cell::new(Rect::new(0, 0, 332, 28)),
            local_position: Cell::new(Point::new(0, 0)),
            vertical_placement: Cell::new(VerticalPlacement::Absolute),
            horizontal_placement: Cell::new(HorizontalPlacement::Absolute),
            margin: Cell::new(Thickness::default()),
            children: RefCell::new(vec![]),
            selector: CloneCell::new(Selector::new(Some("combo-box"))),
            pressed: Cell::new(false),
            activated: Cell::new(false),
            offset: Cell::new(Point::new(4, 4)),
            selected: Cell::new(None),
            entries: RefCell::new(vec![]),
            text: CloneCell::new(String::new()),
            flyout_height: Cell::new(0),
            toggle_icon,
            toggle_icon_active,
        })
    }

    pub fn selected(&self) -> i32 {
        if let Some(selected) = self.selected.get() {
            return selected as i32;
        };

        -1
    }

    pub fn push(&self, text: &str) {
        let rect = self.rect().get();
        let entry = Entry::new(text, self.entries.borrow().len() as u32);
        entry.rect.set(Rect::new(
            rect.x + 1,
            rect.y + rect.height as i32 * (self.entries.borrow().len() as i32 + 1),
            rect.width - 2,
            rect.height,
        ));
        entry.text_offset(self.offset.get().x, self.offset.get().y);
        self.flyout_height
            .set(self.flyout_height.get() + rect.height);

        self.entries.borrow_mut().push(entry);

        if self.entries.borrow().len() == 1 {
            self.change_selection(0);
        }
    }

    pub fn pop(&self) -> String {
        if let Some(entry) = self.entries.borrow_mut().pop() {
            self.change_selection(0);
            return entry.text.get();
        }

        String::new()
    }

    pub fn change_selection(&self, i: u32) {
        if let Some(index) = self.selected.get() {
            if let Some(entry) = self.entries.borrow().get(index as usize) {
                entry.active.set(false)
            }
        }

        self.selected.set(Some(i));

        if let Some(index) = self.selected.get() {
            if let Some(entry) = self.entries.borrow().get(index as usize) {
                entry.active.set(true);
                self.text.set(entry.text.get());
            }
        }
    }

    pub fn text_offset(&self, x: i32, y: i32) -> &Self {
        self.offset.set(Point::new(x, y));
        self
    }
}

impl Style for ComboBox {
    fn selector(&self) -> &CloneCell<Selector> {
        &self.selector
    }
}

impl Widget for ComboBox {
    fn rect(&self) -> &Cell<Rect> {
        &self.rect
    }

    fn local_position(&self) -> &Cell<Point> {
        &self.local_position
    }

    fn vertical_placement(&self) -> &Cell<VerticalPlacement> {
        &self.vertical_placement
    }

    fn horizontal_placement(&self) -> &Cell<HorizontalPlacement> {
        &self.horizontal_placement
    }

    fn margin(&self) -> &Cell<Thickness> {
        &self.margin
    }

    fn draw(&self, renderer: &mut dyn Renderer, focused: bool, theme: &Theme) {
        let rect = self.rect.get();
        let activated = self.activated.get();
        let offset = self.offset.get();

        // draw flyout
        if activated {
            //TODO: set this selector as the child of self.selector
            let selector = Selector::new(Some("combo-box-flyout"));

            let flyout_rect = Rect::new(
                rect.x,
                rect.y + rect.height as i32 - 2,
                rect.width,
                self.flyout_height.get() + 2,
            );
            draw_box(renderer, flyout_rect, theme, &selector);

            // draw entries
            for entry in self.entries.borrow().iter() {
                let point = Point::new(entry.rect.get().x, entry.rect.get().y);

                if point.y >= rect.y
                    && point.y + rect.height as i32 <= flyout_rect.y + flyout_rect.height as i32
                {
                    entry.draw(renderer, focused, theme);
                }
            }
        }

        // draw the combobox
        let mut selector = self.selector.get();

        if activated {
            selector = selector.with_pseudo_class("active");
        }

        if focused {
            selector = selector.with_pseudo_class("focus");
        }

        draw_box(renderer, rect, theme, &selector);

        // draw toggle indicator
        //TODO: set this selector as the child of self.selector
        selector = Selector::new(Some("combo-box-toggle"));

        if activated {
            selector = selector.with_pseudo_class("active");
        }

        let toggle_size = rect.height - 2 * offset.y as u32;

        let toggle_rect = Rect::new(
            rect.x + rect.width as i32 - toggle_size as i32 - offset.y,
            rect.y + offset.y,
            toggle_size,
            toggle_size,
        );

        draw_box(renderer, toggle_rect, theme, &selector);

        // draw the toggle icon
        if activated {
            if let Some(ref icon) = *self.toggle_icon_active.borrow() {
                icon.position(toggle_rect.x, toggle_rect.y);
                icon.draw(renderer, focused, theme)
            }
        } else {
            if let Some(ref icon) = *self.toggle_icon.borrow() {
                icon.position(toggle_rect.x, toggle_rect.y);
                icon.draw(renderer, focused, theme)
            }
        }

        // draw selected text
        let mut point = Point::new(rect.x + offset.x, rect.y + rect.height as i32 / 2 - 8);
        for c in self.text.get().chars() {
            if point.x + 8 <= rect.x + rect.width as i32 - toggle_rect.width as i32 - 2 * offset.x {
                renderer.char(
                    point.x,
                    point.y,
                    c,
                    theme.color("color", &"label".into()),
                );
            }
            point.x += 8;
        }
    }

    fn event(&self, event: Event, mut focused: bool, redraw: &mut bool, caught: &mut bool) -> bool {
        let mut ignore_event = false;
        if self.activated.get() {
            for entry in self.entries.borrow().iter() {
                if entry.event(event, focused, redraw, caught) {
                    ignore_event = true;
                    self.change_selection(entry.index);
                    if self.activated.check_set(false) {
                        *redraw = true;
                    }
                }

                if *caught {
                    break;
                }
            }
        }

        match event {
            Event::Mouse { point, left_button, .. } => {
                let rect = self.rect.get();
                if rect.contains(point) {
                    if left_button {
                        self.pressed.set(!self.pressed.get());

                        if self.activated.check_set(true) {
                            focused = true;
                            *redraw = true;
                        }
                    } else {
                        if !self.pressed.get() {
                            if self.activated.check_set(false) {
                                focused = true;
                                *redraw = true;
                            }
                        }
                    }

                    *caught = true;
                } else {
                    if !ignore_event {
                        if left_button {
                            self.pressed.set(false);
                        } else {
                            if !self.pressed.get() {
                                if self.activated.check_set(false) {
                                    *redraw = true;
                                }
                            }
                        }
                    }
                }
            }
            Event::KeyPressed(key_event) if focused => match key_event.scancode {
                orbclient::K_UP => match self.selected.get() {
                    None => {
                        self.change_selection(0);
                        *redraw = true;
                    }
                    Some(i) => if i > 0 {
                        self.change_selection(i - 1);
                        *redraw = true;
                    },
                },
                orbclient::K_DOWN => match self.selected.get() {
                    None => {
                        self.change_selection(0);
                        *redraw = true;
                    }
                    Some(i) => if i < self.entries.borrow().len() as u32 - 1 {
                        self.change_selection(i + 1);
                        *redraw = true;
                    },
                },
                orbclient::K_ENTER => if self.activated.check_set(false) {
                    self.pressed.set(false);
                    *redraw = true;
                },
                _ => {}
            },

            _ => {}
        }

        focused
    }

    fn children(&self) -> &RefCell<Vec<Arc<dyn Widget>>> {
        &self.children
    }

    fn name(&self) -> &str {
        "ComboBox"
    }
}

impl Place for ComboBox {}
