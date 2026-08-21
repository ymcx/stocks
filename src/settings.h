#ifndef SETTINGS_H
#define SETTINGS_H

#include "gio/gio.h"

GSettings *settings_new(void);
void settings_free(GSettings *g_settings);
gchar **get_bookmarks(GSettings *settings);
void append_bookmarks(GSettings *settings, char *bookmark);
void remove_bookmark(GSettings *settings, guint index);
void set_last_stock(GSettings *settings, char *stock);
char *settings_get_active_symbol(GSettings *settings);
void settings_set_active_symbol(GSettings *settings,char*symbol);

#endif
