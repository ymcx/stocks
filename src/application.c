#include "src/application.h"
#include "gio/gio.h"
#include "gtk/gtk.h"
#include "src/settings.h"
#include "src/window.h"

#define APPLICATION_ID "org.gnome.Stocks"

GtkApplication *application_new(GSettings *g_settings) {
  gchar **bookmarks = settings_get_bookmarks(g_settings);

  GtkApplication *app =
      gtk_application_new(APPLICATION_ID, G_APPLICATION_DEFAULT_FLAGS);
  g_signal_connect(app, "activate", G_CALLBACK(window_activate), bookmarks);

  return app;
}

int application_run(GtkApplication *gtk_application, int argc, char **argv) {
  GApplication *g_application = G_APPLICATION(gtk_application);
  int status = g_application_run(g_application, argc, argv);

  return status;
}

void application_free(GtkApplication *gtk_application) {
  g_object_unref(gtk_application);
}
