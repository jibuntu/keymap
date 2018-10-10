#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include<linux/input.h>

#define push 1
#define leave 0

int write_key_event(int fd, int keycode, int isPushing){
  struct input_event ev;
  
  ev.type = EV_KEY;
  ev.code = keycode;
  ev.value = isPushing;

  return write(fd, &ev, sizeof(struct input_event));
}

int key_pushed(int fd){
  struct input_event ev;

  while(1){
    read(fd, &ev, sizeof(struct input_event));
    if(ev.value == 1){
      break;
    }
  }

  return ev.code;
}

void main(void){
  int fd;
  struct input_event ev;

  fd = open("/dev/input/event22", O_RDWR);
  if(fd == -1){
    perror("open()");
    return;
  }

  for(int i = 0; i < 5; i++){
//    int code = key_pushed(fd);
//    printf("push! %d\n", code);
    write_key_event(fd, KEY_A, push);
    write_key_event(fd, KEY_A, leave);
  }

  close(fd);
}