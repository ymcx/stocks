#ifndef API_CURL_H
#define API_CURL_H

#include <curl/curl.h>
#include <stddef.h>

typedef struct {
  char *data;
  size_t size;
} Response;

size_t api_curl_write(char *input, size_t size, size_t count, void *output);
CURL *api_curl_new(void);
void api_curl_free(CURL *curl);
char *api_curl_run(CURL *curl, const char *url);

#endif
