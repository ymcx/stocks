#ifndef APPLICATION_H
#define APPLICATION_H

#include "gtk/gtk.h"
#include <stdbool.h>

GtkApplication *application_new(GSettings *g_settings);
int application_run(GtkApplication *gtk_application,int argc,char**argv) ;
void application_free(GtkApplication *gtk_application);

#endif
