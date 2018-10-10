#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <linux/input.h>

void main(void){
  int fd;
  struct input_event ev;

  fd = open("/dev/input/event22", O_RDWR);
  if(fd == -1){
    perror("open()");
    return;
  }

  sleep(1);
  printf("start sleep\n");
  ioctl(fd, EVIOCGRAB, 1);
  sleep(2);
  ioctl(fd, EVIOCGRAB, 0);
  printf("end sleep\n");

  close(fd);
}