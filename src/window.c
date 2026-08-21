#include "gtk/gtk.h"
#include "gtk/gtkshortcut.h"
#include "src/models/stock.h"
#include <adwaita.h>

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

static AdwNavigationView *navigation_view;


void activate(GtkApplication *app, gpointer user_data) {
  Stock *dataa = user_data;
  gchar *ticker = dataa->symbol;

  printf("%s\n", ticker);

  /* -----------------------------------------------------------
   * Window
   * ----------------------------------------------------------- */

  AdwApplicationWindow *window =
      ADW_APPLICATION_WINDOW(adw_application_window_new(GTK_APPLICATION(app)));

  gtk_window_set_title(GTK_WINDOW(window), "Window");
  gtk_window_set_default_size(GTK_WINDOW(window), 700, 700);

  /* -----------------------------------------------------------
   * Navigation view
   * ----------------------------------------------------------- */

  navigation_view = ADW_NAVIGATION_VIEW(adw_navigation_view_new());

  /* -----------------------------------------------------------
   * Main page
   * ----------------------------------------------------------- */

  GtkWidget *main_content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);

  /* Title row */

  GtkWidget *title_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 12);



  GtkWidget *text = gtk_label_new(ticker);

  gtk_widget_add_css_class(text, "title-2");

  gtk_box_append(GTK_BOX(title_row), text);

  gtk_box_append(GTK_BOX(main_content), title_row);

  /* Chart */

  GtkWidget *chart = gtk_drawing_area_new();

  gtk_widget_set_size_request(chart, 600, 600);

  gtk_drawing_area_set_draw_func(GTK_DRAWING_AREA(chart), draw_chart, user_data,
                                 NULL);

  gtk_box_append(GTK_BOX(main_content), chart);

  /* -----------------------------------------------------------
   * Main header
   * ----------------------------------------------------------- */

  AdwHeaderBar *main_header = ADW_HEADER_BAR(adw_header_bar_new());

  GtkWidget *add_button=
    gtk_button_new_with_label("+");


  adw_header_bar_pack_start(
    main_header,
    add_button
);



  /* -----------------------------------------------------------
   * Main toolbar view
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
   * Add pages to navigation view
   * ----------------------------------------------------------- */

  adw_navigation_view_add(navigation_view, main_page);


  /* -----------------------------------------------------------
   * Navigation view becomes window content
   * ----------------------------------------------------------- */

  adw_application_window_set_content(window, GTK_WIDGET(navigation_view));

  /* Show window */

  gtk_window_present(GTK_WINDOW(window));
}
