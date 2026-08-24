#include "src/ui/window.h"
#include "gtk/gtk.h"
#include "src/api/yahoo.h"
#include "src/literals.h"
#include "src/settings.h"
#include <adwaita.h>

void ui_window_draw_chart(GtkDrawingArea *_ G_GNUC_UNUSED, cairo_t *cr, int width, int height,
                          gpointer user_data) {
  Stock *stock = user_data;
  Price **data = stock->prices;
  size_t data_len = stock->prices_length;

  double max = data[0]->close;
  double min = data[0]->close;
  for (size_t i = 1; i < data_len; i++) {
    double close = data[i]->close;
    if (close > max) {
      max = close;
    }
    if (close < min) {
      min = close;
    }
  }

  /* Leave some room around the chart */
  // double margin = 40;
  double chart_width = width;
  double chart_height = height;

  /* Background */
  cairo_set_source_rgb(cr, 0.1, 0.1, 0.1);
  cairo_paint(cr);

  /* Axes */
  cairo_set_source_rgb(cr, 0.5, 0.5, 0.5);
  cairo_set_line_width(cr, 1);

  cairo_move_to(cr, 0, 0);
  cairo_line_to(cr, 0, height);
  cairo_line_to(cr, width, height);

  cairo_stroke(cr);

  /* Data line */
  cairo_set_source_rgb(cr, 0.2, 0.6, 1.0);
  cairo_set_line_width(cr, 3);

  for (size_t i = 0; i < data_len; i++) {
    double di = i;
    double close = data[i]->close;

    double x = di / (data_len - 1) * chart_width;

    double y = chart_height - (close - min) / (max - min) * chart_height;

    if (i == 0)
      cairo_move_to(cr, x, y);
    else
      cairo_line_to(cr, x, y);
  }

  cairo_stroke(cr);

  for (size_t i = 0; i < data_len; i++) {
    double di = i;
    double close = data[i]->close;

    double x = di / (data_len - 1) * chart_width;

    double y = chart_height - (close - min) / (max - min) * chart_height;

    cairo_arc(cr, x, y, 5, 0, 2 * G_PI);

    cairo_set_source_rgb(cr, 1.0, 1.0, 1.0);
    cairo_fill(cr);
  }
}

AdwNavigationView *navigation_view;

void ui_window_callback_add_stock(GtkButton *_ G_GNUC_UNUSED, gpointer user_data) {
  AdwApplicationWindow *window = user_data;
  AdwAlertDialog *dialog = ADW_ALERT_DIALOG(adw_alert_dialog_new(
      "Add stock", "Enter the ticker symbol you want to add."));

  adw_alert_dialog_add_response(dialog, "cancel", "Cancel");

  adw_alert_dialog_add_response(dialog, "add", "Add");

  adw_alert_dialog_set_response_appearance(dialog, "add",
                                           ADW_RESPONSE_SUGGESTED);

  adw_alert_dialog_set_default_response(dialog, "add");

  adw_alert_dialog_set_close_response(dialog, "cancel");

  adw_dialog_present(ADW_DIALOG(dialog), GTK_WIDGET(window));
}

AdwToolbarView *ui_window_create_toolbar(AdwApplicationWindow *window) {
  GMenu *menu = g_menu_new();
  g_menu_append(menu, "Preferences", "app.preferences");
  g_menu_append(menu, "About", "app.about");
  g_menu_append(menu, "Quit", "app.quit");

  AdwHeaderBar *main_header = ADW_HEADER_BAR(adw_header_bar_new());

  if (window) {
    GtkWidget *add_button = gtk_button_new_from_icon_name("list-add-symbolic");
    g_signal_connect(add_button, "clicked",
                     G_CALLBACK(ui_window_callback_add_stock), window);
    adw_header_bar_pack_start(main_header, add_button);
  }

  GtkWidget *menu_button = gtk_menu_button_new();

  gtk_menu_button_set_icon_name(GTK_MENU_BUTTON(menu_button),
                                "open-menu-symbolic");

  gtk_menu_button_set_menu_model(GTK_MENU_BUTTON(menu_button),
                                 G_MENU_MODEL(menu));

  adw_header_bar_pack_end(main_header, menu_button);

  AdwToolbarView *main_toolbar = ADW_TOOLBAR_VIEW(adw_toolbar_view_new());

  adw_toolbar_view_add_top_bar(main_toolbar, GTK_WIDGET(main_header));

  return main_toolbar;
}

AdwNavigationPage *ui_window_create_stock_page(Stock *stock) {
  GtkWidget *content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);

  GtkWidget *chart = gtk_drawing_area_new();
  gtk_widget_set_size_request(chart, 600, 600);
  gtk_drawing_area_set_draw_func(GTK_DRAWING_AREA(chart), ui_window_draw_chart,
                                 stock, NULL);

  gtk_box_append(GTK_BOX(content), chart);

  AdwToolbarView *toolbar = ui_window_create_toolbar(NULL);
  adw_toolbar_view_set_content(toolbar, content);

  AdwNavigationPage *page = ADW_NAVIGATION_PAGE(
      adw_navigation_page_new(GTK_WIDGET(toolbar), stock->symbol));

  return page;
}

void ui_window_callback_open_stock_page(GtkButton *_ G_GNUC_UNUSED, gpointer user_data) {
  char *symbol = user_data;

  Stock *stock = api_yahoo_get_stock(symbol);

  AdwNavigationPage *page = ui_window_create_stock_page(stock);
  adw_navigation_view_push(navigation_view, page);
}

AdwNavigationPage *
ui_window_create_bookmarks_page(GSettings *settings,
                                AdwApplicationWindow *window) {
  gchar **bookmarks = settings_get_bookmarks(settings);
  int bookmarks_length = g_strv_length(bookmarks);

  GtkWidget *main_content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);

  GtkWidget *list = gtk_list_box_new();
  for (int i = 0; i < bookmarks_length; i++) {
    char *label = bookmarks[i];

    GtkWidget *button = gtk_button_new_with_label(label);

    gtk_list_box_append(GTK_LIST_BOX(list), button);

    g_signal_connect(button, "clicked",
                     G_CALLBACK(ui_window_callback_open_stock_page), label);
  }

  gtk_box_append(GTK_BOX(main_content), list);

  AdwToolbarView *main_toolbar = ui_window_create_toolbar(window);

  adw_toolbar_view_set_content(main_toolbar, main_content);

  AdwNavigationPage *main_page = ADW_NAVIGATION_PAGE(
      adw_navigation_page_new(GTK_WIDGET(main_toolbar), "Main"));

  adw_navigation_page_set_tag(main_page, "main");

  return main_page;
}

void ui_window_activate(GtkApplication *app, gpointer settingsp) {
  AdwApplicationWindow *window =
      ADW_APPLICATION_WINDOW(adw_application_window_new(GTK_APPLICATION(app)));

  gtk_window_set_title(GTK_WINDOW(window), "Stocks");

  GSettings *settings = settingsp;
  gint width = settings_get_window_width(settings);
  gint height = settings_get_window_height(settings);
  gboolean maximized = settings_get_window_maximized(settings);

  gtk_window_set_default_size(GTK_WINDOW(window), width, height);
  if (maximized) {
    gtk_window_maximize(GTK_WINDOW(window));
  }

  g_settings_bind(settingsp, KEY_WINDOW_MAXIMIZED, window, "maximized",
                  G_SETTINGS_BIND_DEFAULT);
  g_settings_bind(settingsp, KEY_WINDOW_WIDTH, window, "default-width",
                  G_SETTINGS_BIND_DEFAULT);
  g_settings_bind(settingsp, KEY_WINDOW_HEIGHT, window, "default-height",
                  G_SETTINGS_BIND_DEFAULT);

  navigation_view = ADW_NAVIGATION_VIEW(adw_navigation_view_new());
  AdwNavigationPage *main_page =
      ui_window_create_bookmarks_page(settingsp, window);

  adw_navigation_view_add(navigation_view, main_page);

  adw_application_window_set_content(window, GTK_WIDGET(navigation_view));

  gtk_window_present(GTK_WINDOW(window));
}
