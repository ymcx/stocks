#ifndef SETTINGS_H
#define SETTINGS_H

#include "gio/gio.h"

GSettings *settings_new(const gchar *schema_id);
void settings_free(GSettings *settings);
gboolean settings_get_window_maximized(GSettings *settings);
gint settings_get_window_height(GSettings *settings);
gint settings_get_window_width(GSettings *settings);
gchar **settings_get_bookmarks(GSettings *settings);
gboolean settings_set_bookmarks(GSettings *settings, const gchar **bookmarks);
gboolean settings_add_bookmark(GSettings *settings, const gchar *bookmark);
gboolean settings_remove_bookmark(GSettings *settings, const guint index);

#endif
