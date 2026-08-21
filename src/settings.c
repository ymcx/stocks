#include "src/settings.h"
#include "gio/gio.h"
#include "glib-object.h"
#include <stdio.h>

GSettings *settings_new(void) { return g_settings_new("org.gnome.Stocks"); }

void settings_free(GSettings *g_settings) { g_object_unref(g_settings); }

gchar **settings_get_bookmarks(GSettings *settings) {
  return g_settings_get_strv(settings, "bookmarks");
}

void settings_bookmarks_append(GSettings *settings, char *bookmark) {
  gchar **bookmarks = settings_get_bookmarks(settings);
  guint length = g_strv_length(bookmarks);
  bookmarks = g_renew(char *, bookmarks, length + 2);
  bookmarks[length] = g_strdup(bookmark);
  bookmarks[length + 1] = NULL;

  g_settings_set_strv(settings, "bookmarks", (const gchar *const *)bookmarks);
}

void settings_bookmarks_remove(GSettings *settings, guint index) {
  gchar **bookmarks = settings_get_bookmarks(settings);
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

  g_settings_set_strv(settings, "bookmarks", (const gchar *const *)bookmarks);
}
