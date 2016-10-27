#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>

#include <jack/jack.h>

/* jack_client_open */

__thread jack_client_t* jco_ret = NULL;
__thread jack_status_t  jco_ret_status;

__thread char*          jco_passed_client_name = NULL;
__thread char*          jco_passed_server_name = NULL;
__thread jack_options_t jco_passed_options;
__thread size_t         jco_num_calls;

void jco_set_return(jack_client_t* ptrval) { jco_ret = ptrval; }
void jco_set_status_return(jack_status_t stat) { jco_ret_status = stat; }
char* jco_get_passed_client_name() { return jco_passed_client_name; }
char* jco_get_passed_server_name() { return jco_passed_server_name; }
jack_options_t jco_get_passed_options() { return jco_passed_options; }
size_t jco_get_num_calls() { return jco_num_calls; }

jack_client_t* jack_client_open (
    const char *client_name,
    jack_options_t options,
    jack_status_t *status, ...)
{
  jco_num_calls++;
  // printf("called the wrapper num_calls: %zu\n", jco_num_calls);

  free(jco_passed_client_name);
  jco_passed_client_name = malloc(strlen(client_name) + 1);
  assert(jco_passed_client_name);

  strcpy(jco_passed_client_name, client_name);

  va_list ap;
  va_start(ap, status);
  if (options & JackServerName) {
    // we should be expecting an arg, I guess we just explode if we didn't get
    // one?
    char* server_name = va_arg(ap, char*);
    assert(server_name); // maybe this will help sometimes

    free(jco_passed_server_name);
    jco_passed_server_name = malloc(strlen(server_name) + 1);
    assert(jco_passed_server_name);
    strcpy(jco_passed_server_name, server_name);
  }
  va_end(ap);

  jco_passed_options = options;

  *status = jco_ret_status;
  return jco_ret;
}

void jco_setup() {
  jco_ret        = NULL;
  jco_ret_status = 0;

  jco_passed_client_name = NULL;
  jco_passed_options     = JackNullOption;
  jco_num_calls          = 0;
}

void jco_cleanup() {
  jco_ret = NULL;
  jco_ret_status = 0;

  free(jco_passed_client_name);
  jco_passed_client_name = NULL;
  free(jco_passed_server_name);
  jco_passed_server_name = NULL;
  jco_passed_options     = JackNullOption;
  jco_num_calls          = 0;
}

/* jack_get_client_name */

__thread char*          jgcn_return     = NULL;
__thread jack_client_t* jgcn_passed_cl  = NULL;
__thread size_t         jgcn_call_count = 0;

void jgcn_set_return(const char* ret) {
  free(jgcn_return);
  jgcn_return = malloc(strlen(ret) + 1);
  assert(jgcn_return);
  strcpy(jgcn_return, ret);
}

jack_client_t* jgcn_get_passed_client() { return jgcn_passed_cl; }
size_t jgcn_get_num_calls() { return jgcn_call_count; }

char* jack_get_client_name(jack_client_t* client) {
  jgcn_call_count += 1;
  jgcn_passed_cl = client;
  return jgcn_return;
}

void jgcn_setup() {
  free(jgcn_return);
  jgcn_return = NULL;
  jgcn_passed_cl = NULL;
  jgcn_call_count = 0;
}

void jgcn_cleanup() { jgcn_setup(); }
