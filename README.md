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

## Memory layout for AArch64

| 4KB pages + 3 levels                | Size  | Use    |
| ---                                 | ---   | ---    |
| `0000000000000000-0000007fffffffff` | 512GB | user   |
| `ffffff8000000000-ffffffffffffffff` | 512GB | kernel |

| 4KB pages + 4 levels                | Size  | Use    |
| ---                                 | ---   | ---    |
| `0000000000000000-0000ffffffffffff` | 256TB | user   |
| `ffff000000000000-ffffffffffffffff` | 256TB | kernel |

| 64KB pages + 2 levels               | Size  | Use    |
| ---                                 | ---   | ---    |
| `0000000000000000-000003ffffffffff` | 4TB   | user   |
| `fffffc0000000000-ffffffffffffffff` | 4TB   | kernel |

| 64KB pages + 3 levels               | Size  | Use    |
| ---                                 | ---   | ---    |
| `0000000000000000-0000ffffffffffff` | 256TB | user   |
| `ffff000000000000-ffffffffffffffff` | 256TB | kernel |

## References

- https://web.stanford.edu/class/cs140e/
- https://github.com/shacharr/videocoreiv-qpu-driver
- https://github.com/raspberrypi
- http://jaystation2.maisonikkoku.com/
- https://github.com/NuxiNL/cloudabi
