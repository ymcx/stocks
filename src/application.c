#include "src/application.h"
#include "gio/gio.h"
#include "gtk/gtk.h"
#include "src/models/stock.h"
#include "src/services/yahoo.h"
#include "src/settings.h"
#include "src/utils/parse.h"
#include "src/window.h"
#include <stdio.h>

#define APPLICATION_ID "org.gnome.Stocks"

GtkApplication *application_new(GSettings *g_settings) {
  char *symbol = settings_get_active_symbol(g_settings);
  // printf("%s\n",symbol);
  // return NULL;
  char *data = services_yahoo_fetch_data(symbol);
  Stock*stock = utils_parse_stock(data);

  GtkApplication *app = gtk_application_new(APPLICATION_ID, G_APPLICATION_DEFAULT_FLAGS);
  g_signal_connect(app, "activate", G_CALLBACK(activate), stock);

  return app;
}

int application_run(GtkApplication *gtk_application,int argc,char**argv) {
  GApplication *g_application = G_APPLICATION(gtk_application);
  int status = g_application_run(g_application, argc, argv);

  return status;
}

void application_free(GtkApplication *gtk_application) {
  g_object_unref(gtk_application);
}
