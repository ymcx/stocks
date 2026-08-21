#include "gtk/gtk.h"
#include "gtk/gtkshortcut.h"
#include "src/models/stock.h"
#include "src/services/yahoo.h"
#include "src/utils/parse.h"
#include <adwaita.h>
#include <stdio.h>

void draw_chart(GtkDrawingArea *_, cairo_t *cr, int width, int height,
                gpointer user_data) {
  Stock *dataa = user_data;
  Price *data = dataa->prices;
  int data_len = dataa->prices_length;

  double max = data[0].close;
  double min = data[0].close;
  for (int i = 1; i < data_len; i++) {
    if (data[i].close > max)
      max = data[i].close;
    if (data[i].close < min)
      min = data[i].close;
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

  for (int i = 0; i < data_len; i++) {
    double di = i;
    double close = data[i].close;

    double x = di / (data_len - 1) * chart_width;

    double y = chart_height - (close - min) / (max - min) * chart_height;

    if (i == 0)
      cairo_move_to(cr, x, y);
    else
      cairo_line_to(cr, x, y);
  }

  cairo_stroke(cr);

  /* Draw points */
  for (int i = 0; i < data_len; i++) {
    double di = i;
    double close = data[i].close;

    double x = di / (data_len - 1) * chart_width;

    double y = chart_height - (close - min) / (max - min) * chart_height;

    cairo_arc(cr, x, y, 5, 0, 2 * G_PI);

    cairo_set_source_rgb(cr, 1.0, 1.0, 1.0);
    cairo_fill(cr);
  }
}

AdwNavigationView *navigation_view;

static AdwNavigationPage *create_stock_page(const char *ticker,
                                            gpointer user_data) {
  char *ticker_copy = g_strdup(ticker);
  GtkWidget *stock_content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);

  /* Ticker title */

  GtkWidget *text = gtk_label_new(ticker_copy);

  gtk_widget_add_css_class(text, "title-2");

  gtk_box_append(GTK_BOX(stock_content), text);

  /* Chart */

  char *data = services_yahoo_fetch_data(ticker_copy);
  Stock *stock = utils_parse_stock(data);
  GtkWidget *chart = gtk_drawing_area_new();
  gtk_widget_set_size_request(chart, 600, 600);
  gtk_drawing_area_set_draw_func(GTK_DRAWING_AREA(chart), draw_chart, stock,
                                 NULL);

  gtk_box_append(GTK_BOX(stock_content), chart);

  /* Header */

  AdwHeaderBar *stock_header = ADW_HEADER_BAR(adw_header_bar_new());

  /* Toolbar */

  AdwToolbarView *stock_toolbar = ADW_TOOLBAR_VIEW(adw_toolbar_view_new());

  adw_toolbar_view_add_top_bar(stock_toolbar, GTK_WIDGET(stock_header));

  adw_toolbar_view_set_content(stock_toolbar, stock_content);

  /* Navigation page */

  AdwNavigationPage *page = ADW_NAVIGATION_PAGE(
      adw_navigation_page_new(GTK_WIDGET(stock_toolbar), ticker_copy));

  return page;
}

static void open_page_cb(GtkButton *button, gpointer user_data) {
  char *ticker = user_data;
  AdwNavigationPage *page = create_stock_page(ticker, user_data);
  adw_navigation_view_push(navigation_view, page);
}

void activate(GtkApplication *app, gpointer user_data) {
  gchar **bookmarks = malloc(sizeof(gchar *) * 5);
  bookmarks[0] = "INTC";
  bookmarks[1] = "NVDA";
  int bookmarks_length = 2;

  Stock *dataa = user_data;
  gchar *ticker = dataa->symbol;

  /* -----------------------------------------------------------
   * Window
   * ----------------------------------------------------------- */

  AdwApplicationWindow *window =
      ADW_APPLICATION_WINDOW(adw_application_window_new(GTK_APPLICATION(app)));

  gtk_window_set_title(GTK_WINDOW(window), "Stocks");
  gtk_window_set_default_size(GTK_WINDOW(window), 700, 700);

  /* -----------------------------------------------------------
   * Navigation view
   * ----------------------------------------------------------- */

  navigation_view = ADW_NAVIGATION_VIEW(adw_navigation_view_new());

  /* -----------------------------------------------------------
   * Main page
   * ----------------------------------------------------------- */

  GtkWidget *main_content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);

  // GtkWidget *title = gtk_label_new(ticker);

  // gtk_widget_add_css_class(title, "title-2");

  // gtk_box_append(GTK_BOX(main_content), title);

  GtkWidget *list = gtk_list_box_new();
  for (int i = 0; i < bookmarks_length; i++) {
    char *label = bookmarks[i];

    GtkWidget *button = gtk_button_new_with_label(label);

    gtk_list_box_append(GTK_LIST_BOX(list), button);

    g_signal_connect(button, "clicked", G_CALLBACK(open_page_cb), label);
    // g_free(label);
  }

  // GtkWidget *button =
  //     gtk_button_new_with_label("Go to page");

  gtk_box_append(GTK_BOX(main_content), list);

  /* -----------------------------------------------------------
   * Main header
   * ----------------------------------------------------------- */

  GMenu *menu = g_menu_new();

  g_menu_append(menu, "Preferences", "app.preferences");
  g_menu_append(menu, "About", "app.about");
  g_menu_append(menu, "Quit", "app.quit");

  AdwHeaderBar *main_header = ADW_HEADER_BAR(adw_header_bar_new());

  GtkWidget *add_button = gtk_button_new_from_icon_name("list-add-symbolic");

  GtkWidget *menu_button = gtk_menu_button_new();

  gtk_menu_button_set_icon_name(GTK_MENU_BUTTON(menu_button),
                                "open-menu-symbolic");

  gtk_menu_button_set_menu_model(GTK_MENU_BUTTON(menu_button),
                                 G_MENU_MODEL(menu));

  adw_header_bar_pack_start(main_header, add_button);

  adw_header_bar_pack_end(main_header, menu_button);

  /* -----------------------------------------------------------
   * Main toolbar
   * ----------------------------------------------------------- */

  AdwToolbarView *main_toolbar = ADW_TOOLBAR_VIEW(adw_toolbar_view_new());

  adw_toolbar_view_add_top_bar(main_toolbar, GTK_WIDGET(main_header));

  adw_toolbar_view_set_content(main_toolbar, main_content);

  /* -----------------------------------------------------------
   * Main navigation page
   * ----------------------------------------------------------- */

  AdwNavigationPage *main_page = ADW_NAVIGATION_PAGE(
      adw_navigation_page_new(GTK_WIDGET(main_toolbar), "Main"));

  adw_navigation_page_set_tag(main_page, "main");

  /* -----------------------------------------------------------
   * stock page
   * ----------------------------------------------------------- */

  GtkWidget *stock_content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);

  GtkWidget *stock_label = gtk_label_new("stock");

  gtk_box_append(GTK_BOX(stock_content), stock_label);
  GtkWidget *chart = gtk_drawing_area_new();
  gtk_widget_set_size_request(chart, 600, 600);
  gtk_drawing_area_set_draw_func(GTK_DRAWING_AREA(chart), draw_chart, user_data,
                                 NULL);
  gtk_box_append(GTK_BOX(stock_content), chart);

  GtkWidget *title_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 12);
  GtkWidget *text = gtk_label_new(ticker);
  gtk_widget_add_css_class(text, "title-2");
  gtk_box_append(GTK_BOX(title_row), text);
  gtk_box_append(GTK_BOX(stock_content), title_row);

  /* stock header */

  AdwHeaderBar *stock_header = ADW_HEADER_BAR(adw_header_bar_new());

  /* stock toolbar */

  AdwToolbarView *stock_toolbar = ADW_TOOLBAR_VIEW(adw_toolbar_view_new());

  adw_toolbar_view_add_top_bar(stock_toolbar, GTK_WIDGET(stock_header));

  adw_toolbar_view_set_content(stock_toolbar, stock_content);

  /* stock navigation page */

  /* -----------------------------------------------------------
   * Add pages to navigation view
   * ----------------------------------------------------------- */

  adw_navigation_view_add(navigation_view, main_page);

  /* -----------------------------------------------------------
   * Navigation view is the window content
   * ----------------------------------------------------------- */

  adw_application_window_set_content(window, GTK_WIDGET(navigation_view));

  /* -----------------------------------------------------------
   * Show window
   * ----------------------------------------------------------- */

  gtk_window_present(GTK_WINDOW(window));
}
