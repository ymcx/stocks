#ifndef APPLICATION_H
#define APPLICATION_H

#include <adwaita.h>
#include <gio/gio.h>

AdwApplication *application_new(GSettings *settings);
void application_free(AdwApplication *application);
int application_run(AdwApplication *application, int argc, char **argv);

#endif
