#ifndef SETTINGS_H
#define SETTINGS_H

#include "gio/gio.h"

GSettings *settings_new(void);
void settings_free(GSettings *g_settings);
gchar **settings_get_bookmarks(GSettings *settings);
void settings_bookmarks_append(GSettings *settings, char *bookmark);
void settings_bookmarks_remove(GSettings *settings, guint index);
void set_last_stock(GSettings *settings, char *stock);
char *settings_get_active_symbol(GSettings *settings);
void settings_set_active_symbol(GSettings *settings, char *symbol);
gchar **settings_get_bookmarks(GSettings *settings);

#endif
