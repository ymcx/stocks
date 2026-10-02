use crate::{
    APP_ID, APP_NAME, APP_VERSION,
    api::{Range, Stock},
    settings::Settings,
};
use adw::{
    AboutDialog, Application, ApplicationWindow, Bin, Breakpoint, BreakpointCondition,
    BreakpointConditionLengthType, Dialog, EntryRow, HeaderBar, LengthUnit, NavigationPage,
    NavigationSplitView, NavigationView, PreferencesGroup, PreferencesPage, StatusPage,
    ToolbarView,
    gtk::{
        Box, Button, DragSource, DrawingArea, DropTarget, GestureClick, Label, License, ListBox,
        ListBoxRow, MenuButton, Orientation, PolicyType, PopoverMenu, ScrolledWindow,
        WidgetPaintable,
        cairo::Context,
        gdk::{ContentProvider, DragAction, Rectangle},
        gio::{Menu, SimpleAction, SimpleActionGroup},
        glib::{self, ExitCode, Propagation},
        pango::EllipsizeMode,
    },
    prelude::*,
};
use std::{cell::RefCell, rc::Rc, sync::OnceLock};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, Sender},
};

// TODO: save range in settings

pub fn run() -> ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
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

    let navigation_page = NavigationPage::builder()
        .title(APP_NAME)
        .tag("home")
        .child(&toolbar_view)
        .build();

    navigation_page
}

fn create_about_dialog() -> AboutDialog {
    let about_dialog = AboutDialog::builder()
        .application_icon(APP_ID)
        .application_name(APP_NAME)
        .developer_name("ymcx")
        .issue_url("https://github.com/ymcx/stocks/issues")
        .license_type(License::MitX11)
        .version(APP_VERSION)
        .website("https://github.com/ymcx/stocks")
        .build();

    about_dialog
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
    gesture_click.connect_pressed(move |_, _, x, y| {
        let rectangle = Rectangle::new(x as i32, y as i32, 120, 0);
        popover_menu.set_pointing_to(Some(&rectangle));
        popover_menu.popup();
    });
    list_box_row.add_controller(gesture_click);

    let simple_action_group = SimpleActionGroup::new();
    list_box_row.insert_action_group("row", Some(&simple_action_group));

    let simple_action = SimpleAction::new("delete", None);
    simple_action.connect_activate({
        let list_box_row = list_box_row.clone();
        let settings = settings.clone();
        let sidebar = sidebar.clone();
        move |_, _| {
            let index = list_box_row.index() as usize;
            let is_current_or_home = sidebar.selected_row().is_none_or(|i| i == list_box_row);
            if is_current_or_home {
                glib::idle_add_local_once({
                    let sidebar = sidebar.clone();
                    move || {
                        sidebar.unselect_all();
                    }
                });
            }

            sidebar.remove(&list_box_row);
            settings.remove_bookmark(index);
        }
    });
    simple_action_group.add_action(&simple_action);

    let drag_source = DragSource::builder().actions(DragAction::MOVE).build();
    drag_source.connect_prepare({
        let value = list_box_row.to_value();
        move |_, _, _| {
            let content_provider = ContentProvider::for_value(&value);
            Some(content_provider)
        }
    });
    drag_source.connect_drag_begin(move |drag_source, _| {
        let widget_paintable = WidgetPaintable::new(Some(&row_box));
        drag_source.set_icon(Some(&widget_paintable), -15, -15);
    });
    list_box_row.add_controller(drag_source);

    let drop_target = DropTarget::new(ListBoxRow::static_type(), DragAction::MOVE);
    drop_target.connect_drop({
        let sidebar = sidebar.clone();
        let settings = settings.clone();
        move |target, source, _, _| {
            let Ok(source_row) = source.get::<ListBoxRow>() else {
                return false;
            };
            let Some(target_row) = target
                .widget()
                .and_then(|i| i.downcast::<ListBoxRow>().ok())
            else {
                return false;
            };

            let source_index = source_row.index() as usize;
            let target_index = target_row.index() as usize;
            if source_index == target_index {
                return true;
            }

            sidebar.remove(&source_row);
            sidebar.insert(&source_row, target_index as i32);
            settings.reorder_bookmarks(source_index, target_index);

            true
        }
    });
    list_box_row.add_controller(drop_target);

    list_box_row
}

fn create_sidebar_page(
    sidebar: &ListBox,
    settings: &Settings,
    sender: &Sender<Stock>,
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

    sidebar.connect_row_activated({
        let sender = sender.clone();
        let settings = settings.clone();
        move |_, list_box_row| {
            let bookmarks = settings.get_bookmarks();
            let index = list_box_row.index() as usize;
            if let Some(bookmark) = bookmarks.get(index) {
                let bookmark = bookmark.clone();
                let sender = sender.clone();
                let range = Range::OneMonth;
                let runtime = runtime();
                let stock = Stock::fetch_and_send(bookmark, range, sender);
                runtime.spawn(stock);
            }
        }
    });

    for bookmark in settings.get_bookmarks() {
        let button = create_sidebar_button(sidebar, &bookmark, settings);
        sidebar.append(&button);
    }

    let navigation_page = NavigationPage::new(&toolbar_view, APP_NAME);

    navigation_page
}

fn create_add_dialog(settings: &Settings, sidebar: &ListBox) -> Dialog {
    let preferences_page = PreferencesPage::new();

    let preferences_group = PreferencesGroup::new();
    preferences_page.add(&preferences_group);

    // TODO: empty text on dialog open
    let entry_row = EntryRow::builder()
        .title("Stock Symbol")
        .activates_default(true)
        .build();
    entry_row.connect_changed({
        let entry = entry_row.clone();
        move |_| {
            entry.remove_css_class("error");
        }
    });
    preferences_group.add(&entry_row);

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_bottom(90)
        .margin_end(0)
        .margin_start(0)
        .margin_top(0)
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

    // TODO: disable add button if entry is empty
    let button_add = Button::builder()
        .label("_Add")
        .use_underline(true)
        .css_classes(["suggested-action"])
        .build();
    button_add.connect_clicked({
        let dialog = dialog.clone();
        let settings = settings.clone();
        let sidebar = sidebar.clone();
        let entry_row = entry_row.clone();
        move |_| {
            let bookmark = entry_row.text().trim().to_uppercase();
            if bookmark.is_empty() {
                entry_row.add_css_class("error");
                entry_row.grab_focus();
                return;
            }

            let (sender, mut receiver) = mpsc::channel(1);
            let range = Range::OneMonth;
            let runtime = runtime();
            let stock = Stock::fetch_and_send(bookmark.clone(), range, sender);
            runtime.spawn(stock);

            glib::spawn_future_local({
                let dialog = dialog.clone();
                let settings = settings.clone();
                let sidebar = sidebar.clone();
                let entry_row = entry_row.clone();
                async move {
                    let stock = receiver.recv().await;
                    if stock.is_none() {
                        entry_row.add_css_class("error");
                        entry_row.grab_focus();
                        return;
                    }

                    let button = create_sidebar_button(&sidebar, &bookmark, &settings);
                    sidebar.append(&button);
                    settings.add_bookmark(&bookmark);
                    dialog.close();
                }
            });
        }
    });
    header.pack_end(&button_add);
    dialog.set_default_widget(Some(&button_add));

    let button_cancel = Button::builder()
        .label("_Cancel")
        .use_underline(true)
        .build();
    button_cancel.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            dialog.close();
        }
    });
    header.pack_start(&button_cancel);

    dialog.connect_map(move |_| {
        entry_row.set_text("");
        entry_row.grab_focus();
    });

    dialog
}

fn create_stock_page(stock: Stock) -> NavigationPage {
    let (sender, mut receiver) = mpsc::channel(1);
    let price = stock.get_price_string();
    let symbol = stock.symbol.clone();
    let stock = Rc::new(RefCell::new(stock));

    let toolbar_view = ToolbarView::new();
    let navigation_page = NavigationPage::new(&toolbar_view, &symbol);

    let header_bar = HeaderBar::new();
    toolbar_view.add_top_bar(&header_bar);

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .margin_bottom(32)
        .margin_top(32)
        .margin_start(32)
        .margin_end(32)
        .build();
    toolbar_view.set_content(Some(&content));

    let header = Box::builder().orientation(Orientation::Horizontal).build();
    content.append(&header);

    let title = Label::builder()
        .xalign(0.0)
        .hexpand(true)
        .css_classes(["title-2"])
        .label(&symbol)
        .build();
    header.append(&title);

    let price = Label::builder()
        .xalign(1.0)
        .hexpand(true)
        .css_classes(["title-2"])
        .label(&price)
        .build();
    header.append(&price);

    let chart = DrawingArea::builder().hexpand(true).vexpand(true).build();
    chart.set_draw_func({
        let stock = stock.clone();
        move |area, cr, width, height| {
            let quote_close = &stock.borrow().quote_close;
            draw_chart(area, cr, width, height, quote_close);
        }
    });
    content.append(&chart);

    let ranges = Box::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .spacing(8)
        .build();
    for range in Range::VALUES {
        let button = Button::builder().label(range.as_str()).build();
        let symbol = symbol.clone();
        let sender = sender.clone();
        button.connect_clicked(move |_| {
            let symbol = symbol.clone();
            let sender = sender.clone();
            let runtime = runtime();
            let stock = Stock::fetch_and_send(symbol, range, sender);
            runtime.spawn(stock);
        });
        ranges.append(&button);
    }
    let ranges = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::External)
        .vscrollbar_policy(PolicyType::Never)
        .margin_bottom(8)
        .margin_end(8)
        .margin_start(8)
        .margin_top(8)
        .child(&ranges)
        .build();
    content.append(&ranges);

    glib::spawn_future_local(async move {
        while let Some(i) = receiver.recv().await {
            *stock.borrow_mut() = i;
            chart.queue_draw();
        }
    });

    navigation_page
}

// TODO: chart

fn draw_chart(area: &DrawingArea, cr: &Context, width: i32, height: i32, open: &[f64]) {
    if open.is_empty() || width <= 0 || height <= 0 {
        return;
    }

    let width = f64::from(width);
    let height = f64::from(height);
    let padding = 8.0;
    let chart_width = width - 2.0 * padding;
    let chart_height = height - 2.0 * padding;

    if chart_width <= 0.0 || chart_height <= 0.0 {
        return;
    }

    let min = open.iter().copied().fold(f64::INFINITY, f64::min);
    let max = open.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;

    let step = if open.len() > 1 {
        chart_width / (open.len() - 1) as f64
    } else {
        0.0
    };

    let coords: Vec<(f64, f64)> = open
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let x = padding + index as f64 * step;
            let y = if span > 0.0 {
                padding + (1.0 - (value - min) / span) * chart_height
            } else {
                padding + chart_height / 2.0
            };
            (x, y)
        })
        .collect();

    let first = coords[0];
    let last = coords[coords.len() - 1];
    let baseline = height - padding;

    let color = area.color();
    let (red, green, blue) = (
        f64::from(color.red()),
        f64::from(color.green()),
        f64::from(color.blue()),
    );

    cr.move_to(first.0, baseline);
    for &(x, y) in &coords {
        cr.line_to(x, y);
    }
    cr.line_to(last.0, baseline);
    cr.close_path();
    cr.set_source_rgba(red, green, blue, 0.15);
    cr.fill().expect("filling the chart path should succeed");

    cr.move_to(first.0, first.1);
    for &(x, y) in &coords[1..] {
        cr.line_to(x, y);
    }
    cr.set_source_rgba(red, green, blue, 1.0);
    cr.set_line_width(2.0);
    cr.stroke().expect("stroking the chart path should succeed");
}

fn build_ui(application: &Application) {
    let settings = Settings::new();
    let application_window = ApplicationWindow::builder()
        .application(application)
        .title(APP_NAME)
        .default_width(settings.get_window_width())
        .default_height(settings.get_window_height())
        .maximized(settings.get_window_maximized())
        .build();

    application_window.connect_close_request({
        let settings = settings.clone();
        move |application_window| {
            let (width, height) = application_window.default_size();
            let maximized = application_window.is_maximized();
            settings.set_window_width(width);
            settings.set_window_height(height);
            settings.set_window_maximized(maximized);
            Propagation::Proceed
        }
    });

    let (sender, mut receiver) = mpsc::channel(1);

    let split = NavigationSplitView::new();
    application_window.set_content(Some(&split));

    let sidebar = ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .build();
    let sidebar_page = create_sidebar_page(&sidebar, &settings, &sender);
    split.set_sidebar(Some(&sidebar_page));

    let navigation = NavigationView::new();
    let placeholder_page = create_placeholder_page();
    navigation.replace(&[placeholder_page]);
    let navigation_page = NavigationPage::new(&navigation, APP_NAME);
    split.set_content(Some(&navigation_page));

    let breakpoint_condition = BreakpointCondition::new_length(
        BreakpointConditionLengthType::MaxWidth,
        540.0,
        LengthUnit::Sp,
    );
    let breakpoint = Breakpoint::new(breakpoint_condition);
    breakpoint.add_setter(&split, "collapsed", Some(&true.to_value()));
    application_window.add_breakpoint(breakpoint);

    let action_about = SimpleAction::new("about", None);
    action_about.connect_activate({
        let application_window = application_window.clone();
        move |_, _| {
            let about_dialog = create_about_dialog();
            about_dialog.present(Some(&application_window));
        }
    });
    application.add_action(&action_about);

    let action_add_bookmark = SimpleAction::new("add", None);
    action_add_bookmark.connect_activate({
        let sidebar = sidebar.clone();
        let application_window = application_window.clone();
        move |_, _| {
            let add_dialog = create_add_dialog(&settings, &sidebar);
            add_dialog.present(Some(&application_window));
        }
    });
    application.add_action(&action_add_bookmark);

    glib::spawn_future_local(async move {
        while let Some(stock) = receiver.recv().await {
            let stock_page = create_stock_page(stock);
            navigation.replace(&[stock_page]);
            split.set_show_content(true);
        }
    });

    application_window.present();
    sidebar.unselect_all();
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());

    runtime
}
