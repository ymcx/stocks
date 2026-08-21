#include "src/settings.h"
#include "gio/gio.h"
#include "glib-object.h"
#include <stdio.h>

GSettings *settings_new(void) { return g_settings_new("org.gnome.Stocks"); }

void settings_free(GSettings *g_settings) { g_object_unref(g_settings); }

gchar **get_bookmarks(GSettings *settings) {
  return g_settings_get_strv(settings, "bookmarks");
}

void append_bookmarks(GSettings *settings, char *bookmark) {
  gchar **bookmarks = get_bookmarks(settings);
  guint length = g_strv_length(bookmarks);
  bookmarks = g_renew(char *, bookmarks, length + 2);
  bookmarks[length] = g_strdup(bookmark);
  bookmarks[length + 1] = NULL;

  g_settings_set_strv(settings, "bookmarks", (const gchar *const *)bookmarks);
}

void remove_bookmark(GSettings *settings, guint index) {
  gchar **bookmarks = get_bookmarks(settings);
  guint length = g_strv_length(bookmarks);

  if (index >= length) {
    return;
  }

  free(bookmarks[index]);
  bookmarks[index] = NULL;

  for (guint i = length - 2; i >= index; --i) {
    bookmarks[i] = bookmarks[i + 1];
  }
  bookmarks[length - 1] = NULL;
  // bookmarks=g_renew(char*, bookmarks, length+0);

  // bookmarks[length]     = g_strdup (bookmark);
  // bookmarks[length + 1] = NULL;

  // gchar**bookmarks_new=g_new(char*, length+2);
  // for (guint i=0; i<length;++i){
  //   bookmarks_new[i] = bookmarks[i];
  // }
  // bookmarks_new[length]=bookmark;

  g_settings_set_strv(settings, "bookmarks", (const gchar *const *)bookmarks);
}

void set_last_stock(GSettings *settings, char *stock) {
  // g_settings_set_string(settings, "last", (const gchar *)stock);
}

char *settings_get_active_symbol(GSettings *settings) {
  return g_settings_get_string(settings, "last-symbol");
}

void settings_set_active_symbol(GSettings *settings, char *symbol) {
  g_settings_set_string(settings, "last-symbol", symbol);
  g_settings_sync();
}

gchar **settings_get_bookmarks(GSettings *settings) {
  return g_settings_get_strv(settings, "bookmarks");
}
