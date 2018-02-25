# LainOS

[QEMU Raspberry Pi 3](https://github.com/bztsrc/qemu-raspi3)

## Layers

1. kernel layer
2. root single user layer
3. userpsace containers layer

In the root layer live system demons.
ABI/syscalls is not stable here.

In the user layer live common programms.
Only cloudabi is available here.


## File Hierarchy

- `/bin/` contains executables
- `/etc/` configuration files
- `/lib/` shared libraries
- `/sys/` virtual file system
- `/usr/` home directories
- `/var/` contains variable files

## References

- https://web.stanford.edu/class/cs140e/
- https://github.com/shacharr/videocoreiv-qpu-driver
- https://github.com/raspberrypi
- http://jaystation2.maisonikkoku.com/
- https://github.com/NuxiNL/cloudabi
