#include "src/application.h"
#include "src/literals.h"
#include "src/ui/window.h"
#include <adwaita.h>
#include <gio/gio.h>

AdwApplication *application_new(GSettings *settings) {
  AdwApplication *application =
      adw_application_new(ID, G_APPLICATION_DEFAULT_FLAGS);
  if (!application) {
    return NULL;
  }

  g_signal_connect(application, "activate", G_CALLBACK(ui_window_activate),
                   settings);

  return application;
}

void application_free(AdwApplication *application) {
  g_object_unref(application);
}

int application_run(AdwApplication *application, int argc, char **argv) {
  return g_application_run(G_APPLICATION(application), argc, argv);
}
