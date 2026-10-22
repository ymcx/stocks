use crate::{
    APP_ID, APP_NAME, APP_VERSION,
    api::{Range, Stock},
    settings::Settings,
};
use adw::{
    AboutDialog, Application, ApplicationWindow, Bin, Breakpoint, BreakpointCondition,
    BreakpointConditionLengthType, Dialog, EntryRow, HeaderBar, LengthUnit, NavigationPage,
    NavigationSplitView, NavigationView, PreferencesGroup, PreferencesPage, StatusPage, Toggle,
    ToggleGroup, ToolbarView,
    gtk::{
        Align, Box, Button, DragSource, DrawingArea, DropTarget, EventControllerMotion,
        GestureClick, Label, License, ListBox, ListBoxRow, MenuButton, Orientation, Overlay,
        PopoverMenu, ScrolledWindow, WidgetPaintable,
        cairo::{Context, FontSlant, FontWeight, LinearGradient},
        gdk::{ContentProvider, DragAction, RGBA, Rectangle},
        gio::{Menu, SimpleAction, SimpleActionGroup},
        glib::{self, ExitCode, Propagation},
        pango::EllipsizeMode,
    },
    prelude::*,
};
use jiff::{Timestamp, tz::TimeZone};
use std::{cell::RefCell, f64::consts::PI, rc::Rc, sync::OnceLock};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, Sender},
};

#[derive(Clone, Default)]
struct State {
    cursor: Option<(f64, f64)>,
    range: Option<Range>,
    stock: Option<Stock>,
    symbol: Option<String>,
}

impl State {
    fn new(settings: &Settings) -> State {
        let cursor = None;
        let stock = None;
        let range = settings.get_last_range();
        let symbol = settings.get_last_stock();

        State {
            cursor,
            range,
            stock,
            symbol,
        }
    }
}

pub fn run() -> ExitCode {
    let application = Application::builder().application_id(APP_ID).build();
    application.connect_activate(build_ui);
    application.run()
}

fn create_placeholder_page() -> NavigationPage {
    let header_bar = HeaderBar::new();
    let status_page = StatusPage::builder()
        .title(APP_NAME)
        .description("Select a stock from the sidebar")
        .icon_name("view-list-symbolic")
        .build();

    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar);
    toolbar_view.set_content(Some(&status_page));

    NavigationPage::builder()
        .title(APP_NAME)
        .tag("home")
        .child(&toolbar_view)
        .build()
}

fn create_about_dialog() -> AboutDialog {
    AboutDialog::builder()
        .application_icon(APP_ID)
        .application_name(APP_NAME)
        .developer_name("ymcx")
        .issue_url("https://github.com/ymcx/stocks/issues")
        .license_type(License::MitX11)
        .version(APP_VERSION)
        .website("https://github.com/ymcx/stocks")
        .build()
}

fn create_sidebar_button(sidebar: &ListBox, bookmark: &str, settings: &Settings) -> ListBoxRow {
    let list_box_row = ListBoxRow::new();

    let row_box = Box::builder()
        .margin_bottom(8)
        .margin_end(8)
        .margin_start(8)
        .margin_top(8)
        .build();
    list_box_row.set_child(Some(&row_box));

    let label = Label::builder()
        .ellipsize(EllipsizeMode::End)
        .label(bookmark)
        .build();
    row_box.append(&label);

    // TODO: Automatically select Delete by default
    let menu = Menu::new();
    menu.append(Some("Delete"), Some("row.delete"));

    let popover_menu = PopoverMenu::builder()
        .menu_model(&menu)
        .has_arrow(false)
        .build();
    popover_menu.set_parent(&list_box_row);

    let gesture_click = GestureClick::builder().button(3).build();
    let popover_menu_value = popover_menu.clone();
    gesture_click.connect_pressed(move |_, _, x, y| {
        let rectangle = Rectangle::new(x as i32, y as i32, 120, 0);
        popover_menu_value.set_pointing_to(Some(&rectangle));
        popover_menu_value.popup();
    });
    list_box_row.add_controller(gesture_click);

    let simple_action_group = SimpleActionGroup::new();
    list_box_row.insert_action_group("row", Some(&simple_action_group));

    let simple_action = SimpleAction::new("delete", None);
    let list_box_row_value = list_box_row.clone();
    let settings_value = settings.clone();
    let sidebar_value = sidebar.clone();
    simple_action.connect_activate(move |_, _| {
        let index = list_box_row_value.index() as usize;
        let is_current_or_home = sidebar_value
            .selected_row()
            .is_none_or(|i| i == list_box_row_value);

        if is_current_or_home {
            let sidebar_value_value = sidebar_value.clone();
            glib::idle_add_local_once(move || {
                sidebar_value_value.unselect_all();
            });
        }

        sidebar_value.remove(&list_box_row_value);
        settings_value.remove_bookmark(index);
    });
    simple_action_group.add_action(&simple_action);

    let drag_source = DragSource::builder().actions(DragAction::MOVE).build();
    let list_box_row_value = list_box_row.clone();
    drag_source.connect_prepare(move |_, _, _| {
        let content_provider = ContentProvider::for_value(&list_box_row_value.to_value());
        Some(content_provider)
    });
    let row_box_value = row_box.clone();
    drag_source.connect_drag_begin(move |drag_source, _| {
        let widget_paintable = WidgetPaintable::new(Some(&row_box_value));
        drag_source.set_icon(Some(&widget_paintable), -15, -15);
    });
    list_box_row.add_controller(drag_source);

    let drop_target = DropTarget::new(ListBoxRow::static_type(), DragAction::MOVE);
    let sidebar_value = sidebar.clone();
    let settings_value = settings.clone();
    drop_target.connect_drop(move |target, source, _, _| {
        let Ok(source_row) = source.get::<ListBoxRow>() else {
            return false;
        };
        let Some(target_row) = target
            .widget()
            .and_then(|i| i.downcast::<ListBoxRow>().ok())
        else {
            return false;
        };

        let source_index = source_row.index();
        let target_index = target_row.index();
        if source_index == target_index {
            return true;
        }

        sidebar_value.remove(&source_row);
        sidebar_value.insert(&source_row, target_index);
        settings_value.reorder_bookmarks(source_index as usize, target_index as usize);

        true
    });
    list_box_row.add_controller(drop_target);

    list_box_row
}

fn create_sidebar_page(
    sidebar: &ListBox,
    settings: &Settings,
    sender: &Sender<Stock>,
    state: &Rc<RefCell<State>>,
) -> NavigationPage {
    let toolbar_view = ToolbarView::new();

    let header_bar = HeaderBar::new();
    toolbar_view.add_top_bar(&header_bar);

    let bin = Bin::builder().child(sidebar).build();

    let scrolled_window = ScrolledWindow::builder().child(&bin).build();
    toolbar_view.set_content(Some(&scrolled_window));

    let menu = Menu::new();
    menu.append(Some("About"), Some("app.about"));

    let add_button = Button::builder()
        .icon_name("bookmark-new-symbolic")
        .action_name("app.add")
        .build();
    header_bar.pack_start(&add_button);

    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .build();
    header_bar.pack_end(&menu_button);

    let state_value = state.clone();
    let sender_value = sender.clone();
    let settings_value = settings.clone();
    sidebar.connect_row_activated(move |_, list_box_row| {
        let bookmarks = settings_value.get_bookmarks();
        let index = list_box_row.index();
        let range = state_value.borrow().range.unwrap_or_default();
        if let Some(bookmark) = bookmarks.get(index as usize) {
            let future = Stock::fetch_and_send(bookmark.clone(), range, sender_value.clone());
            runtime().spawn(future);
        }
    });

    for bookmark in settings.get_bookmarks() {
        let button = create_sidebar_button(sidebar, &bookmark, settings);
        sidebar.append(&button);
    }

    NavigationPage::new(&toolbar_view, APP_NAME)
}

fn create_add_dialog(settings: &Settings, sidebar: &ListBox) -> Dialog {
    let preferences_page = PreferencesPage::new();

    let preferences_group = PreferencesGroup::new();
    preferences_page.add(&preferences_group);

    let button_add = Button::builder()
        .label("_Add")
        .use_underline(true)
        .sensitive(false)
        .css_classes(["suggested-action"])
        .build();

    let entry_row = EntryRow::builder()
        .title("Stock Symbol")
        .activates_default(true)
        .build();
    let entry_row_value = entry_row.clone();
    let button_add_value = button_add.clone();
    entry_row.connect_changed(move |_| {
        let empty = entry_row_value.text().is_empty();
        button_add_value.set_sensitive(!empty);
        entry_row_value.remove_css_class("error");
    });
    preferences_group.add(&entry_row);

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(90)
        .build();
    content.append(&preferences_page);

    let header = HeaderBar::builder()
        .show_end_title_buttons(false)
        .show_start_title_buttons(false)
        .build();

    let toolbar = ToolbarView::builder().content(&content).build();
    toolbar.add_top_bar(&header);

    let dialog = Dialog::builder()
        .title("Add Bookmark")
        .content_width(640)
        .child(&toolbar)
        .build();

    let dialog_value = dialog.clone();
    let settings_value = settings.clone();
    let sidebar_value = sidebar.clone();
    let entry_row_value = entry_row.clone();
    button_add.connect_clicked(move |_| {
        let bookmark = entry_row_value.text().trim().to_uppercase();
        if bookmark.is_empty() || settings_value.find_bookmark(&bookmark).is_some() {
            entry_row_value.add_css_class("error");
            entry_row_value.grab_focus();
            return;
        }

        let (sender, mut receiver) = mpsc::channel(1);
        let range = Range::default();
        let future = Stock::fetch_and_send(bookmark.clone(), range, sender);
        runtime().spawn(future);

        let dialog_value_value = dialog_value.clone();
        let settings_value_value = settings_value.clone();
        let sidebar_value_value = sidebar_value.clone();
        let entry_row_value_value = entry_row_value.clone();
        let bookmark_value = bookmark.clone();
        glib::spawn_future_local(async move {
            let stock = receiver.recv().await;
            if stock.is_none() {
                entry_row_value_value.add_css_class("error");
                entry_row_value_value.grab_focus();
                return;
            }

            let button =
                create_sidebar_button(&sidebar_value_value, &bookmark_value, &settings_value_value);
            sidebar_value_value.append(&button);
            settings_value_value.add_bookmark(&bookmark_value);
            dialog_value_value.close();
        });
    });
    header.pack_end(&button_add);
    dialog.set_default_widget(Some(&button_add));

    let button_cancel = Button::builder()
        .label("_Cancel")
        .use_underline(true)
        .build();
    let dialog_value = dialog.clone();
    button_cancel.connect_clicked(move |_| {
        dialog_value.close();
    });
    header.pack_start(&button_cancel);

    let entry_row_value = entry_row.clone();
    dialog.connect_map(move |_| {
        entry_row_value.set_text("");
        entry_row_value.grab_focus();
    });

    dialog
}

fn create_stock_page_header_title(long_name: &str) -> Box {
    let container = Box::builder().build();

    let title = Label::builder()
        .css_classes(["title-1"])
        .ellipsize(EllipsizeMode::End)
        .label(long_name)
        .build();
    container.append(&title);

    container
}

fn stock_quote_high(quote_close: &[f64]) -> f64 {
    quote_close
        .iter()
        .copied()
        .max_by(f64::total_cmp)
        .unwrap_or_default()
}

fn stock_quote_low(quote_close: &[f64]) -> f64 {
    quote_close
        .iter()
        .copied()
        .min_by(f64::total_cmp)
        .unwrap_or_default()
}

fn stock_quote_change(quote_close: &[f64], start: Option<usize>, end: Option<usize>) -> f64 {
    let start = start.unwrap_or(0);
    let end = end.unwrap_or(quote_close.len() - 1);

    let first = quote_close.get(start).copied().unwrap_or_default();
    let last = quote_close.get(end).copied().unwrap_or_default();

    last - first
}

fn stock_quote_change_percent(
    quote_close: &[f64],
    start: Option<usize>,
    end: Option<usize>,
) -> f64 {
    let start = start.unwrap_or(0);
    let end = end.unwrap_or(quote_close.len() - 1);

    let first = quote_close.get(start).copied().unwrap_or_default();
    let last = quote_close.get(end).copied().unwrap_or_default();
    if first == 0.0 {
        return 0.0;
    }

    (last / first - 1.0) * 100.0
}

fn create_stock_page_header_price(
    quote_close: &[f64],
    regular_market_price: f64,
    currency: &str,
) -> (Box, Label) {
    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let price = format!("{:.2} {}", regular_market_price, currency);
    let price = Label::builder()
        .css_classes(["title-1"])
        .label(price)
        .build();
    container.append(&price);

    let color = if stock_quote_change(quote_close, None, None) < 0.0 {
        "red"
    } else {
        "blue"
    };
    let change = format!(
        "{:+.2} ({:+.2}%)",
        stock_quote_change(quote_close, None, None),
        stock_quote_change_percent(quote_close, None, None)
    );
    let change = Label::builder()
        .css_classes([color, "title-1"])
        .ellipsize(EllipsizeMode::End)
        .label(change)
        .build();
    container.append(&change);

    (container, change)
}

fn create_stock_page_header_metadata_item(label: &str, value: f64, decimals: bool) -> (Box, Label) {
    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let label = Label::builder()
        .css_classes(["dimmed", "heading"])
        .label(label)
        .build();
    container.append(&label);

    let decimals = if decimals { 2 } else { 0 };
    let value = format!("{:.decimals$}", value);
    let value_label = Label::builder()
        .css_classes(["heading"])
        .label(value)
        .build();
    container.append(&value_label);

    (container, value_label)
}

fn create_stock_page_header_metadata(
    quote_close: &[f64],
    regular_market_volume: i64,
) -> (Box, Label, Label) {
    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let (high, high_value) =
        create_stock_page_header_metadata_item("High", stock_quote_high(quote_close), true);
    container.append(&high);

    let (low, low_value) =
        create_stock_page_header_metadata_item("Low", stock_quote_low(quote_close), true);
    container.append(&low);

    let (volume, _) =
        create_stock_page_header_metadata_item("Volume", regular_market_volume as f64, false);
    container.append(&volume);

    (container, high_value, low_value)
}

fn update_stock_page_header_left(
    quote_close: &[f64],
    change_label: &Label,
    high_label: &Label,
    low_label: &Label,
) {
    let change = stock_quote_change(quote_close, None, None);
    let percent = stock_quote_change_percent(quote_close, None, None);
    let color = if change < 0.0 { "red" } else { "blue" };

    change_label.set_css_classes(&[color, "title-1"]);
    change_label.set_text(&format!("{:+.2} ({:+.2}%)", change, percent));

    high_label.set_text(&format!("{:.2}", stock_quote_high(quote_close)));
    low_label.set_text(&format!("{:.2}", stock_quote_low(quote_close)));
}

fn create_stock_page_header(
    long_name: &str,
    currency: &str,
    quote_close: &[f64],
    regular_market_price: f64,
    regular_market_volume: i64,
) -> (Box, Label, Label, Label) {
    let header = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(16)
        .margin_end(16)
        .spacing(8)
        .build();

    let title = create_stock_page_header_title(long_name);
    header.append(&title);

    let (price, change) =
        create_stock_page_header_price(quote_close, regular_market_price, currency);
    header.append(&price);

    let (metadata, high, low) =
        create_stock_page_header_metadata(quote_close, regular_market_volume);
    header.append(&metadata);

    (header, change, high, low)
}

fn create_stock_page_chart_infobox() -> (Bin, Label, Label, Label) {
    let infobox = Bin::builder()
        .css_classes(["card", "infobox"])
        .margin_bottom(16)
        .margin_end(16)
        .margin_start(16)
        .margin_top(16)
        .halign(Align::Start)
        .valign(Align::Start)
        .visible(false)
        .can_target(false)
        .build();

    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(16)
        .margin_end(16)
        .margin_start(16)
        .margin_top(16)
        .spacing(8)
        .build();
    infobox.set_child(Some(&container));

    let time = Label::builder()
        .css_classes(["dimmed", "heading"])
        .xalign(0.0)
        .build();
    container.append(&time);

    let subcontainer = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();
    container.append(&subcontainer);

    let quote = Label::builder()
        .css_classes(["heading"])
        .halign(Align::Start)
        .build();
    subcontainer.append(&quote);

    let spacer = Box::builder().hexpand(true).build();
    subcontainer.append(&spacer);

    let change = Label::builder()
        .css_classes(["heading"])
        .halign(Align::End)
        .build();
    subcontainer.append(&change);

    (infobox, time, quote, change)
}

fn create_stock_page_chart(
    state: &Rc<RefCell<State>>,
    change: &Label,
    high: &Label,
) -> (Overlay, DrawingArea) {
    let time_zone = TimeZone::system();
    let overlay = Overlay::new();

    let chart = DrawingArea::builder().vexpand(true).hexpand(true).build();
    overlay.set_child(Some(&chart));

    let (infobox, infobox_time, infobox_quote, infobox_change) = create_stock_page_chart_infobox();
    overlay.add_overlay(&infobox);

    let motion = EventControllerMotion::new();
    chart.add_controller(motion.clone());

    let chart_value = chart.clone();
    let state_value = state.clone();
    let infobox_value = infobox.clone();
    let infobox_quote_value = infobox_quote.clone();
    let infobox_change_value = infobox_change.clone();
    let infobox_time_value = infobox_time.clone();
    let time_zone_value = time_zone.clone();
    motion.connect_motion(move |_, x, y| {
        let Some(stock) = state_value.borrow().stock.clone() else {
            return;
        };

        if stock.quote_close.is_empty() {
            return;
        }

        let width = chart_value.width() as f64;
        let amount = stock.quote_close.len() as f64 - 1.0;
        let i = (x / width * amount).round().min(amount) as usize;

        let change_percent = stock_quote_change_percent(&stock.quote_close, None, Some(i));
        let quote = stock.quote_close[i];
        let time = stock.timestamp[i];

        let Ok(time) = Timestamp::from_second(time) else {
            return;
        };

        let color = if change_percent < 0.0 { "red" } else { "blue" };
        infobox_change_value.set_css_classes(&[color, "heading"]);

        let quote_length = quote.abs().floor().to_string().len();
        let decimals = if quote_length < 4 { 2 } else { 0 };
        infobox_quote_value.set_text(&format!("{:.decimals$}", quote));

        let change_percent_length = change_percent.abs().floor().to_string().len();
        let decimals = if change_percent_length < 4 { 2 } else { 0 };
        infobox_change_value.set_text(&format!("{:+.decimals$}%", change_percent));

        let time = time_zone_value
            .to_datetime(time)
            .strftime("%m/%d/%Y %I:%M %p")
            .to_string();
        infobox_time_value.set_text(&time);

        state_value.borrow_mut().cursor = Some((x, y));
        infobox_value.set_visible(true);
        chart_value.queue_draw();
    });

    let chart_value = chart.clone();
    let infobox_value = infobox.clone();
    let state_value = state.clone();
    motion.connect_leave(move |_| {
        state_value.borrow_mut().cursor = None;
        infobox_value.set_visible(false);
        chart_value.queue_draw();
    });

    let state_value = state.clone();
    let change_value = change.clone();
    let high_value = high.clone();
    chart.set_draw_func(move |_, cr, width, height| {
        let Some(stock) = state_value.borrow().stock.clone() else {
            return;
        };

        let quote_close = stock.quote_close;
        let cursor = state_value.borrow().cursor;
        let color_change = change_value.color();
        let color_foreground = high_value.color();

        draw_chart(
            cr,
            width,
            height,
            &quote_close,
            &cursor,
            color_change,
            color_foreground,
        );
    });

    (overlay, chart)
}

fn create_stock_page_ranges(state: &Rc<RefCell<State>>, sender: &Sender<Stock>) -> ToggleGroup {
    let ranges = ToggleGroup::builder()
        .margin_start(16)
        .margin_end(16)
        .homogeneous(true)
        .build();

    for range in [
        Range::OneDay,
        Range::FiveDays,
        Range::OneMonth,
        Range::SixMonths,
        Range::Ytd,
        Range::OneYear,
        Range::FiveYears,
        Range::Max,
    ] {
        let toggle = Toggle::builder()
            .label(range.as_str())
            .name(range.as_api_str())
            .build();
        ranges.add(toggle);
    }

    let range = state.borrow().range.unwrap_or_default();
    ranges.set_active_name(Some(range.as_api_str()));

    let state_value = state.clone();
    let sender_value = sender.clone();
    ranges.connect_notify_local(Some("active-name"), move |toggle_group, _| {
        let Some(range) = toggle_group.active_name().and_then(|i| Range::parse(&i)) else {
            return;
        };

        let Some(symbol) = state_value.borrow().symbol.clone() else {
            return;
        };

        state_value.borrow_mut().range = Some(range);

        let future = Stock::fetch_and_send(symbol, range, sender_value.clone());
        runtime().spawn(future);
    });

    ranges
}

fn create_stock_page(state: &Rc<RefCell<State>>) -> NavigationPage {
    let Some(symbol) = state.borrow().symbol.clone() else {
        return create_placeholder_page();
    };

    let Some(stock) = state.borrow().stock.clone() else {
        return create_placeholder_page();
    };

    let (sender, mut receiver) = mpsc::channel(1);
    let navigation_page = NavigationPage::builder().title(symbol).build();

    let toolbar_view = ToolbarView::new();
    navigation_page.set_child(Some(&toolbar_view));

    let header_bar = HeaderBar::new();
    toolbar_view.add_top_bar(&header_bar);

    let content = Box::builder()
        .css_classes(["numeric"])
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(16)
        .build();
    toolbar_view.set_content(Some(&content));

    let (header, change, high, low) = create_stock_page_header(
        &stock.long_name,
        &stock.currency,
        &stock.quote_close,
        stock.regular_market_price,
        stock.regular_market_volume,
    );
    content.append(&header);

    let (chart, chart_area) = create_stock_page_chart(state, &change, &high);
    content.append(&chart);

    let ranges = create_stock_page_ranges(state, &sender);
    content.append(&ranges);

    let state_value = state.clone();
    glib::spawn_future_local(async move {
        while let Some(stock) = receiver.recv().await {
            update_stock_page_header_left(&stock.quote_close, &change, &high, &low);
            state_value.borrow_mut().stock = Some(stock.clone());
            state_value.borrow_mut().symbol = Some(stock.symbol);
            chart_area.queue_draw();
        }
    });

    navigation_page
}

fn get_quote_levels_step(estimate: f64) -> f64 {
    let magnitude = 10f64.powf(estimate.log10().floor());
    let normalized = estimate / magnitude;
    let step = if normalized < 1.5 {
        1.0
    } else if normalized < 3.0 {
        2.0
    } else if normalized < 7.0 {
        5.0
    } else {
        10.0
    };

    step * magnitude
}

fn get_decimals_for_step(step: f64) -> usize {
    let mut i = step;
    let mut decimals = 0;
    while i != i.round() {
        i *= 10.0;
        decimals += 1;
    }

    decimals
}

fn get_quote_levels(min: f64, max: f64, height: f64) -> (Vec<f64>, usize) {
    let ideal_amount = height / 100.0;
    let step_estimate = (max - min) / ideal_amount;
    let step = get_quote_levels_step(step_estimate);

    let mut levels = Vec::new();
    let mut i = (min / step).ceil() * step;
    while i <= max {
        levels.push(i);
        i += step;
    }

    let decimals = get_decimals_for_step(step);

    (levels, decimals)
}

fn draw_chart(
    cr: &Context,
    width: i32,
    height: i32,
    quote_close: &[f64],
    cursor: &Option<(f64, f64)>,
    color_change: RGBA,
    color_foreground: RGBA,
) {
    if quote_close.is_empty() || width == 0 || height == 0 {
        return;
    }

    let width = width as f64;
    let height = height as f64;
    let step = width / usize::max(quote_close.len() - 1, 1) as f64;

    let min = quote_close.iter().copied().fold(f64::INFINITY, f64::min);
    let max = quote_close
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;

    let coordinates: Vec<(f64, f64)> = quote_close
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            let x = step * i as f64;
            let y = height - (value - min) / span * height;
            (x, y)
        })
        .collect();
    let first = coordinates[0];
    let last = coordinates[coordinates.len() - 1];

    cr.move_to(first.0, height);
    for &(x, y) in &coordinates {
        cr.line_to(x, y);
    }
    cr.line_to(last.0, height);
    cr.close_path();
    let gradient = LinearGradient::new(0.0, 0.0, 0.0, height);
    gradient.add_color_stop_rgba(
        0.0,
        color_change.red() as f64,
        color_change.green() as f64,
        color_change.blue() as f64,
        0.1,
    );
    gradient.add_color_stop_rgba(
        1.0,
        color_change.red() as f64,
        color_change.green() as f64,
        color_change.blue() as f64,
        0.0,
    );
    cr.set_source(gradient).unwrap();
    cr.fill().unwrap();

    cr.move_to(first.0, first.1);
    for &(x, y) in &coordinates[1..] {
        cr.line_to(x, y);
    }
    cr.set_source_rgba(
        color_change.red() as f64,
        color_change.green() as f64,
        color_change.blue() as f64,
        1.0,
    );
    cr.set_line_width(2.0);
    cr.stroke().unwrap();

    // TODO: add x labels
    // TODO: fix the right margin of y labels
    let (levels, decimals) = get_quote_levels(min, max, height);
    if !levels.is_empty() {
        let text = format!("{:.decimals$}", levels[levels.len() - 1]);
        let extents = cr.text_extents(&text).unwrap();
        for level in levels {
            let x = width - 24.0 - extents.width();
            let y = height - (level - min) / span * height - extents.height() / 2.0;
            let text = format!("{:.decimals$}", level);
            cr.set_source_rgba(
                color_foreground.red() as f64,
                color_foreground.green() as f64,
                color_foreground.blue() as f64,
                0.5,
            );
            cr.select_font_face("Adwaita Sans", FontSlant::Normal, FontWeight::Normal);
            cr.set_font_size(16.0);
            cr.move_to(x, y);
            cr.show_text(&text).unwrap();
        }
    }

    let &Some((x, _)) = cursor else {
        return;
    };

    let amount = (coordinates.len() - 1) as f64;
    let index = (x / width * amount).round().min(amount) as usize;
    let (x, y) = coordinates[index];

    cr.set_line_width(1.0);
    cr.set_dash(&[4.0, 4.0], 0.0);
    cr.move_to(x, 0.0);
    cr.line_to(x, height);
    cr.move_to(0.0, y);
    cr.line_to(width, y);
    cr.stroke().unwrap();
    cr.set_dash(&[], 0.0);

    cr.set_source_rgba(
        color_foreground.red() as f64,
        color_foreground.green() as f64,
        color_foreground.blue() as f64,
        1.0,
    );
    cr.arc(x, y, 5.0, 0.0, 2.0 * PI);
    cr.fill().unwrap();
}

fn build_ui(application: &Application) {
    let settings = Settings::new();
    let (sender, mut receiver) = mpsc::channel(1);
    let state = Rc::new(RefCell::new(State::new(&settings)));
    let application_window = ApplicationWindow::builder()
        .application(application)
        .title(APP_NAME)
        .default_width(settings.get_window_width())
        .default_height(settings.get_window_height())
        .maximized(settings.get_window_maximized())
        .width_request(640)
        .height_request(480)
        .build();

    let state_value = state.clone();
    let settings_value = settings.clone();
    application_window.connect_close_request(move |application_window| {
        let (width, height) = application_window.default_size();
        let maximized = application_window.is_maximized();
        let symbol = state_value.borrow().symbol.clone();
        let range = state_value.borrow().range;

        settings_value.set_window_width(width);
        settings_value.set_window_height(height);
        settings_value.set_window_maximized(maximized);
        settings_value.set_last_stock(symbol);
        settings_value.set_last_range(range);

        Propagation::Proceed
    });

    let split = NavigationSplitView::new();
    application_window.set_content(Some(&split));

    let sidebar = ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .build();
    let sidebar_page = create_sidebar_page(&sidebar, &settings, &sender, &state);
    split.set_sidebar(Some(&sidebar_page));

    let navigation = NavigationView::new();
    let placeholder_page = create_placeholder_page();
    navigation.push(&placeholder_page);

    let navigation_page = NavigationPage::new(&navigation, APP_NAME);
    split.set_content(Some(&navigation_page));

    let state_value = state.clone();
    let settings_value = settings.clone();
    let sidebar_value = sidebar.clone();
    let split_value = split.clone();
    let placeholder_page_value = placeholder_page.clone();
    navigation.connect_popped(move |navigation, _| {
        let Some(visible_page) = navigation.visible_page() else {
            return;
        };

        if visible_page == placeholder_page_value {
            state_value.borrow_mut().symbol = None;
            sidebar_value.unselect_all();
            split_value.set_show_content(false);
            return;
        }

        let symbol = visible_page.title().to_string();
        state_value.borrow_mut().symbol = Some(symbol.clone());

        if let Some(index) = settings_value.find_bookmark(&symbol) {
            sidebar_value.select_row(sidebar_value.row_at_index(index as i32).as_ref());
        }
    });

    let breakpoint_condition = BreakpointCondition::new_length(
        BreakpointConditionLengthType::MaxWidth,
        800.0,
        LengthUnit::Sp,
    );
    let breakpoint = Breakpoint::new(breakpoint_condition);
    breakpoint.add_setter(&split, "collapsed", Some(&true.to_value()));
    application_window.add_breakpoint(breakpoint);

    let action_about = SimpleAction::new("about", None);
    let application_window_value = application_window.clone();
    action_about.connect_activate(move |_, _| {
        let about_dialog = create_about_dialog();
        about_dialog.present(Some(&application_window_value));
    });
    application.add_action(&action_about);

    let action_add_bookmark = SimpleAction::new("add", None);
    let settings_value = settings.clone();
    let sidebar_value = sidebar.clone();
    let application_window_value = application_window.clone();
    action_add_bookmark.connect_activate(move |_, _| {
        let add_dialog = create_add_dialog(&settings_value, &sidebar_value);
        add_dialog.present(Some(&application_window_value));
    });
    application.add_action(&action_add_bookmark);

    if let Some(symbol) = settings.get_last_stock() {
        let range = settings.get_last_range().unwrap_or_default();
        let future = Stock::fetch_and_send(symbol, range, sender);
        runtime().spawn(future);
    }

    let state_value = state.clone();
    let sidebar_value = sidebar.clone();
    let split_value = split.clone();
    let navigation_value = navigation.clone();
    let settings_value = settings.clone();
    glib::spawn_future_local(async move {
        while let Some(stock) = receiver.recv().await {
            state_value.borrow_mut().stock = Some(stock.clone());
            state_value.borrow_mut().symbol = Some(stock.symbol.clone());

            let stock_page = create_stock_page(&state_value);
            if let Some(visible_page) = navigation.visible_page()
                && visible_page.title() == stock_page.title()
            {
                continue;
            }

            navigation_value.push(&stock_page);
            split_value.set_show_content(true);

            if let Some(index) = settings_value.find_bookmark(&stock.symbol) {
                sidebar_value.select_row(sidebar_value.row_at_index(index as i32).as_ref());
            }
        }
    });

    application_window.present();
    sidebar.unselect_all();
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();

    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}
