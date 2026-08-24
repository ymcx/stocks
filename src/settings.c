#include "src/settings.h"
#include "gio/gio.h"
#include "src/literals.h"

GSettings *settings_new(const gchar *schema_id) {
  return g_settings_new(schema_id);
}

void settings_free(GSettings *settings) { g_object_unref(settings); }

gboolean settings_get_window_maximized(GSettings *settings) {
  return g_settings_get_boolean(settings, KEY_WINDOW_MAXIMIZED);
}

gint settings_get_window_height(GSettings *settings) {
  return g_settings_get_int(settings, KEY_WINDOW_HEIGHT);
}

gint settings_get_window_width(GSettings *settings) {
  return g_settings_get_int(settings, KEY_WINDOW_WIDTH);
}

gchar **settings_get_bookmarks(GSettings *settings) {
  return g_settings_get_strv(settings, KEY_BOOKMARKS);
}

gboolean settings_set_bookmarks(GSettings *settings, const gchar **bookmarks) {
  if (!bookmarks) {
    return FALSE;
  }

  return g_settings_set_strv(settings, KEY_BOOKMARKS, bookmarks);
}

gboolean settings_add_bookmark(GSettings *settings, const gchar *bookmark) {
  if (!bookmark) {
    return FALSE;
  }

  gchar **bookmarks = settings_get_bookmarks(settings);
  if (!bookmarks) {
    return FALSE;
  }

  const guint bookmarks_length = g_strv_length(bookmarks);

  bookmarks = g_realloc(bookmarks, sizeof(gchar *) * (bookmarks_length + 2));
  bookmarks[bookmarks_length] = g_strdup(bookmark);
  bookmarks[bookmarks_length + 1] = NULL;

  const gboolean status =
      settings_set_bookmarks(settings, (const gchar **)bookmarks);

  g_strfreev(bookmarks);

  return status;
}

gboolean settings_remove_bookmark(GSettings *settings, const guint index) {
  gchar **bookmarks = settings_get_bookmarks(settings);
  if (!bookmarks) {
    return FALSE;
  }

  const guint bookmarks_length = g_strv_length(bookmarks);
  if (bookmarks_length <= index) {
    g_strfreev(bookmarks);
    return FALSE;
  }

  g_free(bookmarks[index]);
  for (guint i = index; i < bookmarks_length - 1; ++i) {
    bookmarks[i] = bookmarks[i + 1];
  }
  bookmarks[bookmarks_length - 1] = NULL;

  const gboolean status =
      settings_set_bookmarks(settings, (const gchar **)bookmarks);

  g_strfreev(bookmarks);

  return status;
}
