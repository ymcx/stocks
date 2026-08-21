#ifndef WINDOW_H
#define WINDOW_H

#include "gtk/gtk.h"

void draw_chart(GtkDrawingArea *_, cairo_t *cr, int width, int height,
                gpointer user_data);
void activate(GtkApplication *app, gpointer user_data);

#endif
