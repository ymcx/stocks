#ifndef WINDOW_H
#define WINDOW_H

#include "gtk/gtk.h"

void window_draw_chart(GtkDrawingArea *_, cairo_t *cr, int width, int height,
                       gpointer user_data);
void window_activate(GtkApplication *app, gpointer user_data);

#endif
