#ifndef UI_WINDOW_H
#define UI_WINDOW_H

#include "adwaita.h"
#include "gtk/gtk.h"
#include "src/model/stock.h"

void ui_window_draw_chart(GtkDrawingArea *_, cairo_t *cr, int width, int height, gpointer user_data);
void ui_window_callback_add_stock(GtkButton *_, gpointer user_data);
AdwToolbarView *ui_window_create_toolbar(AdwApplicationWindow *window);
AdwNavigationPage *ui_window_create_stock_page(Stock *stock);
void ui_window_callback_open_stock_page(GtkButton *_, gpointer user_data);
AdwNavigationPage *ui_window_create_bookmarks_page(GSettings*settings, AdwApplicationWindow *window) ;
void ui_window_activate(GtkApplication *app, gpointer bookmarks);

#endif
