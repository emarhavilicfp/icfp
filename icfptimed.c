#include <sys/wait.h>
#include <signal.h>
#include <unistd.h>
#include <stdio.h>

void _handler(int a) {}

int main(int argc, char **argv)
{
	int pid;
	struct sigaction act;
	
	act.sa_flags = 0;
	act.sa_mask = 0;
	act.sa_handler = _handler;
	sigaction(SIGALRM, &act, NULL);
	
	pid = fork();
	if (pid == 0) {
		int rv = execv(argv[1], argv + 1);
		if (rv < 0)
			perror("execv");
	} else if (pid > 0) {
		int rv;
		alarm(3);
		rv = wait(NULL);
		if (rv > 0)
			return;
		kill(pid, 2);
		fprintf(stderr, "Your time has ended!!  SIGINT!\n");
		
		alarm(10);
		rv = wait(NULL);
		if (rv > 0)
			return;
		kill(pid, 9);
		fprintf(stderr, "Shit, and goodbye!\n");
	}
	return 0;
}