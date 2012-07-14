#include <signal.h>
#include <string.h>

static volatile int done = 0;

static void
signal_handler(int sig)
{
    done = 1;
}

void
signal_init()
{
    struct sigaction act;
    memset(&act, 0, sizeof(act));
    act.sa_handler = signal_handler;
    sigaction(SIGINT, &act, NULL);
}

int
signal_received()
{
    return done;
}
