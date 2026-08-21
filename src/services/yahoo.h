#ifndef SERVICES_YAHOO_H
#define SERVICES_YAHOO_H

#include <stddef.h>

typedef struct {
  char *data;
  size_t size;
} Response;

size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp);
char *services_yahoo_fetch_data(char *symbol);

#endif
