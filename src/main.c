#include "gio/gio.h"
#include "gtk/gtk.h"
#include "src/application.h"
#include "src/settings.h"
#include <stdlib.h>

int main(int argc,char**argv) {
  GSettings *g_settings = settings_new();
  if (!g_settings) {
    return EXIT_FAILURE;
  }

  GtkApplication *gtk_application = application_new(g_settings);
  if (!gtk_application) {
    settings_free(g_settings);
    return EXIT_FAILURE;
  }

  int status = application_run(gtk_application,argc,argv);

  application_free(gtk_application);
  settings_free(g_settings);

  return status;
}
