#include "src/application.h"
#include "src/literals.h"
#include "src/settings.h"
#include <adwaita.h>
#include <gio/gio.h>

int main(int argc, char **argv) {
  GSettings *settings = settings_new(ID);
  if (!settings) {
    // settings_new shouldn't return NULL unless we're creating the object by
    // passing a NULL as the schema_id.
    return EXIT_FAILURE;
  }

  AdwApplication *application = application_new(settings);
  if (!application) {
    settings_free(settings);
    return EXIT_FAILURE;
  }

  const int status = application_run(application, argc, argv);

  application_free(application);
  settings_free(settings);

  return status;
}
