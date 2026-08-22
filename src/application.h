#ifndef APPLICATION_H
#define APPLICATION_H

#include "gio/gio.h"
#include "gtk/gtk.h"

GtkApplication *application_new(GSettings *settings);
void application_free(GtkApplication *application);
int application_run(GtkApplication *application, int argc, char **argv);

#endif
