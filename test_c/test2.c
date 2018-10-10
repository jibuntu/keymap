#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <linux/uinput.h>

void emit_event(int fd, int type, int code, int value){
  struct input_event event;
  
  event.type = type;
  event.code = code;
  event.value = value;
  event.time.tv_sec = 0;
  event.time.tv_usec = 0;

  write(fd, &event, sizeof(struct input_event));
}

void emit_key_event(int fd, int code, int value){
  emit_event(fd, EV_KEY, code, value);
  emit_event(fd, EV_SYN, SYN_REPORT, 0);
}

void key_input(int fd, int code){
  emit_key_event(fd, code, 1);
  emit_key_event(fd, code, 0);
}

void main(void){
  struct uinput_setup usetup;

  int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);

  if(fd == -1){
    perror("open");
    return;
  }

  ioctl(fd, UI_SET_EVBIT, EV_KEY);
  for(int i = 0; i < KEY_MAX; ++i){
    ioctl(fd, UI_SET_KEYBIT, i);
  }

  memset(&usetup, 0, sizeof(struct uinput_setup));
  usetup.id.bustype = BUS_USB;
  usetup.id.vendor = 0;
  usetup.id.product = 0;
  strcpy(usetup.name, "Example device");

  ioctl(fd, UI_DEV_SETUP, &usetup);
  ioctl(fd, UI_DEV_CREATE);

  sleep(1);

  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);

  sleep(1);

  ioctl(fd, UI_DEV_DESTROY);
  close(fd);
}