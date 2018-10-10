#include <linux/input.h>

int ioctl_eviocgrab(int fd, int mode){
  return ioctl(fd, EVIOCGRAB, mode);
}