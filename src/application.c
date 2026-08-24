#include "src/application.h"
#include "gio/gio.h"
#include "gtk/gtk.h"
#include "src/literals.h"
#include "src/ui/window.h"

GtkApplication *application_new(GSettings *settings) {
  GtkApplication *application =
      gtk_application_new(ID, G_APPLICATION_DEFAULT_FLAGS);
  if (!application) {
    return NULL;
  }

  g_signal_connect(application, "activate", G_CALLBACK(ui_window_activate),
                   settings);

  return application;
}

void application_free(GtkApplication *application) {
  g_object_unref(application);
}

int application_run(GtkApplication *application, int argc, char **argv) {
  return g_application_run(G_APPLICATION(application), argc, argv);
}
