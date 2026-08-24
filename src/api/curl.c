#include "src/api/curl.h"
#include <curl/curl.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

size_t api_curl_write(char *input, size_t size, size_t count, void *output) {
  Response *response = output;
  if (response == NULL) {
    return 0;
  }

  if (size == 0 || count == 0 || count > SIZE_MAX / size ||
      size * count > SIZE_MAX - response->size - 1) {
    return 0;
  }

  const size_t length = size * count;

  char *data = realloc(response->data, response->size + length + 1);
  if (data == NULL) {
    return 0;
  }
  response->data = data;

  memcpy(response->data + response->size, input, length);
  response->size += length;
  response->data[response->size] = '\0';

  return length;
}

CURL *api_curl_new(void) {
  CURL *curl = curl_easy_init();
  if (!curl) {
    return NULL;
  }

  const char *user_agent = "Mozilla/5.0 "
                           "(Windows NT 6.1; Win64; x64) "
                           "AppleWebKit/537.36 "
                           "(KHTML, like Gecko) "
                           "Chrome/58.0.3029.110 "
                           "Safari/537.36";
  curl_easy_setopt(curl, CURLOPT_USERAGENT, user_agent);
  curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, api_curl_write);
  curl_easy_setopt(curl, CURLOPT_FAILONERROR, 1L);
  curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);

  return curl;
}

void api_curl_free(CURL *curl) {
  if (curl) {
    curl_easy_cleanup(curl);
  }
}

char *api_curl_run(CURL *curl, const char *url) {
  if (curl == NULL || url == NULL) {
    return NULL;
  }

  Response response = {NULL, 0};

  curl_easy_setopt(curl, CURLOPT_URL, url);
  curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);

  const CURLcode status = curl_easy_perform(curl);
  if (status != CURLE_OK) {
    free(response.data);
    return NULL;
  }

  return response.data;
}
