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
#include <stdint.h>

static int (*f_start)(void*, unsigned);
static void (*f_stop)(void);
static void (*f_shot)(const char*);
static void (*f_motion)(double, double);
static void (*f_button)(unsigned, _Bool);
static void (*f_scroll)(double);
static void (*f_touch)(unsigned, unsigned long long, double, double);
static void (*f_key)(unsigned, _Bool);
static void (*f_setwin)(void*);
static void (*f_lost)(void);
static int (*g_rnew)(int, int, int, int, void**); static int (*g_rwin)(void*, void**);
static int out;
// режим окна (HARNESS_WINDOW=1): ANativeWindow из AImageReader, как у SurfaceView; потребитель забирает кадры
static void *g_reader, *g_win; static pthread_mutex_t fm = PTHREAD_MUTEX_INITIALIZER;
static uint8_t* g_frame; static int g_fw, g_fh; static volatile long g_frames;
static int (*r_acquire)(void*, void**); static void (*i_delete)(void*);
static int (*i_plane)(void*, int, uint8_t**, int*); static int (*i_stride)(void*, int, int*);
static int (*i_w)(void*, int*); static int (*i_h)(void*, int*);
static void* drain(void* _) {
  for (;;) {
    void* img = NULL; int st = r_acquire(g_reader, &img);
    if (st != 0 || !img) { usleep(4000); continue; }
    uint8_t* d = NULL; int len = 0, stride = 0, w = 0, h = 0;
    i_plane(img, 0, &d, &len); i_stride(img, 0, &stride); i_w(img, &w); i_h(img, &h);
    pthread_mutex_lock(&fm);
    if (!g_frame || g_fw != w || g_fh != h) { free(g_frame); g_frame = malloc((size_t)w * h * 4); g_fw = w; g_fh = h; }
    for (int y = 0; y < h; y++) memcpy(g_frame + (size_t)y * w * 4, d + (size_t)y * stride, (size_t)w * 4);
    pthread_mutex_unlock(&fm); g_frames++;
    i_delete(img);
  }
  return NULL;
}
static int shot_window(const char* path) {
  pthread_mutex_lock(&fm);
  if (!g_frame) { pthread_mutex_unlock(&fm); return 0; }
  FILE* f = fopen(path, "wb"); fprintf(f, "P6\n%d %d\n255\n", g_fw, g_fh);
  for (int i = 0; i < g_fw * g_fh; i++) fwrite(g_frame + i * 4, 1, 3, f);
  fclose(f); pthread_mutex_unlock(&fm); return 1;
}   // копия stdout до того, как композитор перенаправит 1 и 2 в cosmic-comp.log

static void say(const char* fmt, ...) {
  char b[512]; va_list a; va_start(a, fmt); int n = vsnprintf(b, sizeof b, fmt, a); va_end(a); write(out, b, n);
}
static int rc = -1;
static void* run(void* _) { rc = f_start(g_win, 60000); say("composit: cosmic_start вернул %d\n", rc); return NULL; }

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
  SYM(f_setwin, "cosmic_set_window") SYM(f_lost, "cosmic_window_lost") SYM(f_button, "cosmic_pointer_button") SYM(f_scroll, "cosmic_scroll") SYM(f_touch, "cosmic_touch") SYM(f_key, "cosmic_key")
  (void)home;
  if (getenv("HARNESS_WINDOW")) {
    int W = 2392, H = 1080; const char* hs = getenv("COSMIC_HEADLESS"); if (hs) sscanf(hs, "%dx%d", &W, &H);
    void* m = dlopen("libmediandk.so", RTLD_NOW);
    if (!m) { say("libmediandk: %s\n", dlerror()); return 5; }
    int (*rnew)(int, int, int, int, void**) = dlsym(m, "AImageReader_new");
    int (*rwin)(void*, void**) = dlsym(m, "AImageReader_getWindow");
    g_rnew = rnew; g_rwin = rwin;
    r_acquire = dlsym(m, "AImageReader_acquireNextImage"); i_delete = dlsym(m, "AImage_delete");
    i_plane = dlsym(m, "AImage_getPlaneData"); i_stride = dlsym(m, "AImage_getPlaneRowStride");
    i_w = dlsym(m, "AImage_getWidth"); i_h = dlsym(m, "AImage_getHeight");
    int st = rnew(W, H, 0x1 /* AIMAGE_FORMAT_RGBA_8888 */, 3, &g_reader);
    if (st != 0) { say("AImageReader_new: %d\n", st); return 6; }
    rwin(g_reader, &g_win);
    pthread_t dt; pthread_create(&dt, NULL, drain, NULL);
    say("режим окна: AImageReader %dx%d, окно %p\n", W, H, g_win);
  }
  pthread_t th; pthread_create(&th, NULL, run, NULL);
  char line[600];
  while (fgets(line, sizeof line, stdin)) {
    char c[32] = "", p[512] = ""; double a = 0, b = 0; unsigned long long id = 0; unsigned k = 0, pr = 0;
    if (sscanf(line, "%31s", c) != 1) continue;
    if (!strcmp(c, "wait")) { sscanf(line, "%*s %lf", &a); usleep((useconds_t)(a * 1e6)); }
    else if (!strcmp(c, "shot")) {
      sscanf(line, "%*s %511s", p); unlink(p);
      struct stat st; int ok = 0;
      if (g_win) { ok = shot_window(p); }
      else { f_shot(p); for (int i = 0; i < 100 && !ok; i++) { usleep(100000); ok = stat(p, &st) == 0 && st.st_size > 0; } }
      say("shot %s: %s\n", p, ok ? "ok" : "НЕТ ФАЙЛА");
    }
    else if (!strcmp(c, "motion")) { sscanf(line, "%*s %lf %lf", &a, &b); f_motion(a, b); }
    else if (!strcmp(c, "button")) { sscanf(line, "%*s %u %u", &k, &pr); f_button(k, pr); }
    else if (!strcmp(c, "scroll")) { sscanf(line, "%*s %lf", &a); f_scroll(a); }
    else if (!strcmp(c, "touch")) { sscanf(line, "%*s %u %llu %lf %lf", &k, &id, &a, &b); f_touch(k, id, a, b); }
    else if (!strcmp(c, "key")) { sscanf(line, "%*s %u %u", &k, &pr); f_key(k, pr); }
    else if (!strcmp(c, "lost")) { f_lost(); say("окно потеряно\n"); }
    else if (!strcmp(c, "newwin")) {   // новое окно (как при повторном открытии экрана): новый AImageReader
      void* nr = NULL; void* nw = NULL;
      if (g_rnew && g_rnew(g_fw ? g_fw : 2392, g_fh ? g_fh : 1080, 0x1, 3, &nr) == 0) { g_rwin(nr, &nw); g_reader = nr; f_setwin(nw); say("новое окно %p\n", nw); }
      else say("newwin: нужен режим окна\n");
    }
    else if (!strcmp(c, "quit")) break;
    else say("неизвестная команда: %s", line);
  }
  f_stop(); usleep(1500000);
  say("конец (код композитора %d, кадров от окна: %ld)\n", rc, g_frames);
  _exit(rc > 0 ? rc : 0);   // выход из процесса без разборки потоков композитора
}
