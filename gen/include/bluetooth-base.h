#include <linux/types.h>
#include <linux/ioctl.h>

#ifndef BIT
#define BIT(nr) (1UL << (nr))
#endif
#ifndef __packed
#define __packed __attribute__((packed))
#endif
#ifndef __user
#define __user
#endif
#ifndef __counted_by
#define __counted_by(member)
#endif
#ifndef __nonstring
#define __nonstring
#endif
typedef __u16 sa_family_t;
typedef __u8 u8;
typedef __s16 s16;
typedef __u16 u16;
typedef __s32 s32;
typedef __u32 u32;
typedef struct {
    __u8 b[6];
} __packed bdaddr_t;

#define BDADDR_BREDR 0x00
#define BDADDR_LE_PUBLIC 0x01
#define BDADDR_LE_RANDOM 0x02

#ifndef HCI_MAX_SHORT_NAME_LENGTH
#define HCI_MAX_SHORT_NAME_LENGTH 10
#endif

struct sk_buff {
    unsigned char *data;
};
