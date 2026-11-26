// Безоконный тест-харнес cosmic-comp (bionic, обычный процесс Termux): грузит libcosmic_comp.so, запускает композитор
// в потоке с EGL pbuffer (COSMIC_HEADLESS), читает команды со stdin. Не трогает приложение и сессию Claude.
//
//   XDG_RUNTIME_DIR=/path ./harness [libcosmic_comp.so] <<'EOF'
//   wait 3          — пауза, секунд
//   shot /p/a.ppm   — снимок кадра (PPM)
//   motion X Y | button B P | scroll DY | touch K ID X Y | key CODE P
//   quit
//   EOF
#include <dlfcn.h>
#include <stdarg.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static int (*f_start)(void*, unsigned);
static void (*f_stop)(void);
static void (*f_shot)(const char*);
static void (*f_motion)(double, double);
static void (*f_button)(unsigned, _Bool);
static void (*f_scroll)(double);
static void (*f_touch)(unsigned, unsigned long long, double, double);
static void (*f_key)(unsigned, _Bool);
static int out;   // копия stdout до того, как композитор перенаправит 1 и 2 в cosmic-comp.log

static void say(const char* fmt, ...) {
  char b[512]; va_list a; va_start(a, fmt); int n = vsnprintf(b, sizeof b, fmt, a); va_end(a); write(out, b, n);
}
static int rc = -1;
static void* run(void* _) { rc = f_start(NULL, 60000); say("composit: cosmic_start вернул %d\n", rc); return NULL; }

int main(int argc, char** argv) {
  const char* lib = argc > 1 ? argv[1] : "libcosmic_comp.so";
  const char* home = getenv("HOME");
  out = dup(1);
  if (!getenv("XDG_RUNTIME_DIR")) { say("нужен XDG_RUNTIME_DIR\n"); return 2; }
  setenv("COSMIC_BACKEND", "winit", 1); setenv("COSMIC_DISABLE_XWAYLAND", "1", 1);
  if (!getenv("COSMIC_HEADLESS")) setenv("COSMIC_HEADLESS", "2392x1080", 1);
  void* h = dlopen(lib, RTLD_NOW | RTLD_GLOBAL);
  if (!h) { say("dlopen %s: %s\n", lib, dlerror()); return 3; }
#define SYM(v, n) v = dlsym(h, n); if (!v) { say("нет символа %s\n", n); return 4; }
  SYM(f_start, "cosmic_start") SYM(f_stop, "cosmic_stop") SYM(f_shot, "cosmic_screenshot") SYM(f_motion, "cosmic_pointer_motion")
  SYM(f_button, "cosmic_pointer_button") SYM(f_scroll, "cosmic_scroll") SYM(f_touch, "cosmic_touch") SYM(f_key, "cosmic_key")
  (void)home;
  pthread_t th; pthread_create(&th, NULL, run, NULL);
  char line[600];
  while (fgets(line, sizeof line, stdin)) {
    char c[32] = "", p[512] = ""; double a = 0, b = 0; unsigned long long id = 0; unsigned k = 0, pr = 0;
    if (sscanf(line, "%31s", c) != 1) continue;
    if (!strcmp(c, "wait")) { sscanf(line, "%*s %lf", &a); usleep((useconds_t)(a * 1e6)); }
    else if (!strcmp(c, "shot")) {
      sscanf(line, "%*s %511s", p); unlink(p); f_shot(p);
      struct stat st; int ok = 0;
      for (int i = 0; i < 100 && !ok; i++) { usleep(100000); ok = stat(p, &st) == 0 && st.st_size > 0; }
      say("shot %s: %s\n", p, ok ? "ok" : "НЕТ ФАЙЛА");
    }
    else if (!strcmp(c, "motion")) { sscanf(line, "%*s %lf %lf", &a, &b); f_motion(a, b); }
    else if (!strcmp(c, "button")) { sscanf(line, "%*s %u %u", &k, &pr); f_button(k, pr); }
    else if (!strcmp(c, "scroll")) { sscanf(line, "%*s %lf", &a); f_scroll(a); }
    else if (!strcmp(c, "touch")) { sscanf(line, "%*s %u %llu %lf %lf", &k, &id, &a, &b); f_touch(k, id, a, b); }
    else if (!strcmp(c, "key")) { sscanf(line, "%*s %u %u", &k, &pr); f_key(k, pr); }
    else if (!strcmp(c, "quit")) break;
    else say("неизвестная команда: %s", line);
  }
  f_stop(); usleep(1500000);
  say("конец (код композитора %d)\n", rc);
  _exit(rc > 0 ? rc : 0);   // выход из процесса без разборки потоков композитора
}
