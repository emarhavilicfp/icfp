#include <sys/wait.h>
#include <signal.h>
#include <unistd.h>
#include <stdio.h>

void _handler(int a) {}

int main(int argc, char **argv)
{
	int pid;
	struct sigaction act;
	int inttime = 30;
	int killtime = 5;
	
	act.sa_flags = 0;
	sigemptyset(&act.sa_mask);
	act.sa_handler = _handler;
	sigaction(SIGALRM, &act, NULL);
	
	if (getenv("LONG_RUN"))
		inttime = 150;
	
	pid = fork();
	if (pid == 0) {
		int rv = execv(argv[1], argv + 1);
		if (rv < 0)
			perror("execv");
	} else if (pid > 0) {
		int rv;
		alarm(inttime);
		rv = wait(NULL);
		if (rv > 0)
			return 0;
		kill(pid, 2);
		fprintf(stderr, "Your time has ended!!  SIGINT!\n");
		
		alarm(killtime);
		rv = wait(NULL);
		if (rv > 0)
			return 0;
		kill(pid, 9);
		fprintf(stderr, "Shit, and goodbye!\n");
	}
	return 0;
}
