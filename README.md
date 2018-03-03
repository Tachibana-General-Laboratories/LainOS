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

| 4KB pages + 3 levels                    | Sizeof  | Use    |
| ---                                     | ---     | ---    |
| `00000000_00000000 - 0000007f_ffffffff` | 512GB   | user   |
| `ffffff80_00000000 - ffffffff_ffffffff` | 512GB   | kernel |

| 4KB pages + 4 levels                    | Sizeof  | Use    |
| ---                                     | ---     | ---    |
| `00000000_00000000 - 0000ffff_ffffffff` | 256TB   | user   |
| `ffff0000_00000000 - ffffffff_ffffffff` | 256TB   | kernel |

| 64KB pages + 2 levels                   | Sizeof  | Use    |
| ---                                     | ---     | ---    |
| `00000000_00000000 - 000003ff_ffffffff` | 4TB     | user   |
| `fffffc00_00000000 - ffffffff_ffffffff` | 4TB     | kernel |

| 64KB pages + 3 levels                   | Sizeof  | Use    |
| ---                                     | ---     | ---    |
| `00000000_00000000 - 0000ffff_ffffffff` | 256TB   | user   |
| `ffff0000_00000000 - ffffffff_ffffffff` | 256TB   | kernel |

## References

- https://web.stanford.edu/class/cs140e/
- https://github.com/shacharr/videocoreiv-qpu-driver
- https://github.com/raspberrypi
- http://jaystation2.maisonikkoku.com/
- https://github.com/NuxiNL/cloudabi
