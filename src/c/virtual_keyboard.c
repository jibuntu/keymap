#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <linux/uinput.h>

#define RETURN_WRITE_ERR(result) if(result == -1) { return -1; } 

int emit_event(int fd, int type, int code, int value){
  struct input_event event;
  
  event.type = type;
  event.code = code;
  event.value = value;
  event.time.tv_sec = 0;
  event.time.tv_usec = 0;

  return write(fd, &event, sizeof(struct input_event));
}

int emit_event_sync(int fd, int type, int code, int value){
  RETURN_WRITE_ERR(emit_event(fd, type, code, value));
  RETURN_WRITE_ERR(emit_event(fd, EV_SYN, SYN_REPORT, 0));
  return 0;
}

int emit_key_event(int fd, int code, int value){
  RETURN_WRITE_ERR(emit_event(fd, EV_KEY, code, value));
  RETURN_WRITE_ERR(emit_event(fd, EV_SYN, SYN_REPORT, 0));
  return 0;
}

int key_input(int fd, int code){
  RETURN_WRITE_ERR(emit_key_event(fd, code, 1));
  RETURN_WRITE_ERR(emit_key_event(fd, code, 0));
  return 0;
}

int open_virtual_keyboard(const char *name){
  struct uinput_setup usetup;

  int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);

  if(fd == -1){
    return -1;
  }

  if(ioctl(fd, UI_SET_EVBIT, EV_KEY) == -1) { return -1; };
  for(int i = 0; i < KEY_MAX; ++i){
    if(ioctl(fd, UI_SET_KEYBIT, i) == -1) { return -1; };
  }

  memset(&usetup, 0, sizeof(struct uinput_setup));
  usetup.id.bustype = BUS_USB;
  usetup.id.vendor = 0;
  usetup.id.product = 0;
  strcpy(usetup.name, name);

  if(ioctl(fd, UI_DEV_SETUP, &usetup) == -1){ return -1; };
  if(ioctl(fd, UI_DEV_CREATE) == -1){ return -1; };

  return fd;
}

int close_virtual_keyboard(int fd){
  if(ioctl(fd, UI_DEV_DESTROY) == -1){ return -1; };
  if(close(fd) == -1){ return -1; };
  return 0;
}

void example_main(void){
  struct uinput_setup usetup;

  int fd = open_virtual_keyboard("Example keyboard");
  
  if(fd == -1){
    perror("open_virtual_keyboard");
    return;
  }

  sleep(1);

  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);
  key_input(fd, KEY_A);

  sleep(1);

  close_virtual_keyboard(fd);
}
